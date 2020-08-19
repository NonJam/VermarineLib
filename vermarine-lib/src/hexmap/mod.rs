use crate::tetra::math::Vec2;

#[derive(Copy, Clone, Debug)]
pub struct ChunkPos {
    pub q: i32,
    pub r: i32,
}

impl ChunkPos {
    pub fn new(q: i32, r: i32) -> ChunkPos {
        ChunkPos {
            q, r
        }
    }

    pub fn sparse_index(&self) -> (usize, usize) {
        let wrap = |point| {
            let mut point = point * 2;
            if point < 0 { 
                point *= -1;
                point -= 1;
            }
            point as usize
        };

        (wrap(self.q), wrap(self.r))
    }
}

pub const CHUNK_WIDTH: usize = 16;
pub const CHUNK_HEIGHT: usize = 16;
pub const CHUNK_TOTAL: usize = CHUNK_WIDTH * CHUNK_HEIGHT;
pub struct HexChunk<T> {
    tiles: [Option<T>; CHUNK_WIDTH * CHUNK_HEIGHT],
    pos: ChunkPos,
}

impl<T> HexChunk<T> {
    pub fn new(tiles: [Option<T>; CHUNK_WIDTH * CHUNK_HEIGHT], q: i32, r: i32) -> HexChunk<T> {
        HexChunk {
            tiles,
            pos: ChunkPos::new(q, r),
        }
    }

    pub fn sparse_index(&self) -> (usize, usize) {
        self.pos.sparse_index()
    }

    pub fn set_tile(&mut self, hex: &Hex, tile: T) {
        let axial = hex.to_axial();
        
        if axial.q < 0 || (axial.q as usize) >= CHUNK_WIDTH {
            panic!();
        }
        if axial.r < 0 || (axial.r as usize) >= CHUNK_HEIGHT {
            panic!();
        }

        let (q, r) = (axial.q as usize, axial.r as usize);
        self.tiles[q + r * CHUNK_WIDTH] = Some(tile);
    }

    pub fn get_tile(&self, hex: &Hex) -> Option<&T> {
        let axial = hex.to_axial();
        
        if axial.q < 0 || (axial.q as usize) >= CHUNK_WIDTH {
            panic!();
        }
        if axial.r < 0 || (axial.r as usize) >= CHUNK_HEIGHT {
            panic!();
        }

        let (q, r) = (axial.q as usize, axial.r as usize);
        let tile = self.tiles.get(q + r * CHUNK_WIDTH)?;
        tile.as_ref()
    }

    pub fn get_tile_mut(&mut self, hex: &Hex) -> Option<&mut T> {
        let axial = hex.to_axial();
        
        if axial.q < 0 || (axial.q as usize) >= CHUNK_WIDTH {
            panic!();
        }
        if axial.r < 0 || (axial.r as usize) >= CHUNK_HEIGHT {
            panic!();
        }

        let (q, r) = (axial.q as usize, axial.r as usize);
        let tile = self.tiles.get_mut(q + r * CHUNK_WIDTH)?;
        tile.as_mut()
    }
}

//
// Hex map

pub struct HexMap<T> {
    chunks: Vec<HexChunk<T>>,
    chunks_sparse: Vec<Vec<Option<usize>>>,

    pub get_height: fn(&T) -> u8,

    pub position: Vec2<f32>,
    pub tallest: u8,

    pub hex_width: f32,
    pub hex_height: f32,
    pub hex_vert_step: f32,
    pub hex_depth_step: f32,

    pub wall_vert_step: f32,
    pub wall_vert_offset: f32,
}

impl<T> HexMap<T> {
    pub fn new(
        hex_width: f32,
        hex_height: f32,
        hex_vert_step: f32,
        hex_depth_step: f32,
        wall_vert_offset: f32,
        wall_vert_step: f32,
    ) -> Self {
        HexMap {
            chunks: vec![],
            chunks_sparse: vec![], 

            get_height: |_| 0,

            position: Vec2::zero(),
            tallest: 0,

            hex_width,
            hex_height,
            hex_vert_step,
            hex_depth_step,

            wall_vert_offset,
            wall_vert_step,
        }
    }

    pub fn clear_map(&mut self) {
        self.chunks = vec![];
        self.chunks_sparse = vec![];
        self.tallest = 0;
    }

    pub fn hex_to_chunk(&self, hex: &Hex) -> (ChunkPos, Axial) {
        let axial = hex.to_axial();

        let (q, r) = (axial.q, axial.r);

        let chunk_width = CHUNK_WIDTH as i32;
        let (chunk_q, q_offset) = 
            if q < 0 {
                (
                    ((q + 1) / chunk_width) - 1,
                    ((q + 1) % chunk_width) + (chunk_width - 1),
                )
            } else {
                (
                    q / chunk_width,
                    q % chunk_width,
                )
            };

        let chunk_height = CHUNK_HEIGHT as i32;
        let (chunk_r, r_offset) = 
            if r < 0 {
                (
                    ((r + 1) / chunk_height) - 1,
                    ((r + 1) % chunk_height) + (chunk_height - 1),
                )
            } else {
                (
                    r / chunk_height,
                    r % chunk_height,
                )
            };

        let chunk_pos = ChunkPos::new(chunk_q, chunk_r);
        let hex_pos = Axial::new(q_offset, r_offset);
        
        (chunk_pos, hex_pos)
    }

    pub fn insert_chunk(&mut self, chunk: HexChunk<T>) {
        let (q, r) = chunk.sparse_index();
        let chunk_index = self.chunks.len();
        self.chunks.push(chunk);

        if self.chunks_sparse.len() <= q {
            self.chunks_sparse.resize(q + 1, vec![]);
        }
        if self.chunks_sparse[q].len() <= r {
            self.chunks_sparse[q].resize(r + 1, None);
        }

        self.chunks_sparse[q][r] = Some(chunk_index);
    }

    pub fn does_chunk_exist(&self, pos: ChunkPos) -> bool {
        let (q, r) = pos.sparse_index();
        if let Some(chunks) = self.chunks_sparse.get(q) {
            if let Some(index) = chunks.get(r) {
                if index.is_some() {
                    return true;
                }
            }
        }
        false
    }

    pub fn pixel_to_hex_raw(&mut self, pos: Vec2<f32>, height_offset: f32) -> FractionalAxial {
        let mut pos = pos;
        pos -= Vec2::new(18., 18.);
        pos.x -= self.position.x;
        pos.y -= self.position.y;
        pos.y += height_offset;

        let size_x = self.hex_width / f32::sqrt(3.0);
        // this value is derived by solving for X in:
        // FLOOR_VERT_STEP * R = X * (3.0 / 2.0 * R) 
        // R can be 1 so we can simplify to:
        // FLOOR_VERT_STEP = X * 1.5
        // X = FLOOR_VERT_STEP / 1.5
        let size_y = 18.666666;

        let pos = Vec2::new(
            pos.x / size_x,
            pos.y / size_y,
        );

        let m0 = f32::sqrt(3.0) / 3.0;
        let m1 = -1.0 / 3.0;
        let m2 = 0.0;
        let m3 = 2.0 / 3.0;

        FractionalAxial::new(
            m0 * pos.x + m1 * pos.y, 
            m2 * pos.x + m3 * pos.y
        )
    }

    pub fn pixel_to_hex(&mut self, pos: Vec2<f32>) -> Option<Axial> {
        let mut tallest_height: Option<(u8, Axial)> = None;

        for height in 0..=self.tallest {
            let height_offset = height as f32 * self.hex_depth_step;

            let axial_hex = self.pixel_to_hex_raw(pos, height_offset);
            let cube_hex = axial_hex.to_fractional_cube();
            let axial_hex = cube_hex.to_cube().to_axial();
    
            let tile = if let Some(tile) = self.get_tile(Hex::Axial(axial_hex)) {
                tile
            } else {
                continue;
            };

            if (self.get_height)(&tile) != height {
                continue;
            }

            if tallest_height.is_none() || (self.get_height)(&tile) > tallest_height.unwrap().0 {
                tallest_height = Some(((self.get_height)(&tile), axial_hex));
            }
        }

        if let Some((_, hex)) = tallest_height {
            return Some(hex);
        }
        None
    }

    /// Returns the top left pixel of a bounding box around the hex
    pub fn axial_to_pixel(&self, hex: Axial) -> Vec2<f32> {
        let size_x = self.hex_width / f32::sqrt(3.0);
        // this value is derived by solving for X in:
        // FLOOR_VERT_STEP * R = X * (3.0 / 2.0 * R) 
        // R can be 1 so we can simplify to:
        // FLOOR_VERT_STEP = X * 1.5
        // X = FLOOR_VERT_STEP / 1.5
        let size_y = 18.666666;

        let x = size_x * (f32::sqrt(3.0) * hex.q as f32 + f32::sqrt(3.0) / 2.0 * hex.r as f32);
        let y = size_y * (3.0 / 2.0 * hex.r as f32);

        Vec2::new(
            x + self.position.x,
            y + self.position.y,
        )
    }

    pub fn set_tile(&mut self, hex: Hex, tile: T) {
        let (chunk_pos, axial) = self.hex_to_chunk(&hex);
        if !self.does_chunk_exist(chunk_pos) {
            let array = none_array::create_array::<T, CHUNK_TOTAL>();
            self.insert_chunk(HexChunk::new(array, chunk_pos.q, chunk_pos.r))
        }

        let (q, r) = chunk_pos.sparse_index();
        let index = self.chunks_sparse[q][r].unwrap();
        let chunk = &mut self.chunks[index];
        chunk.set_tile(&axial.to_hex(), tile);
    }

    pub fn get_tile(&self, hex: Hex) -> Option<&T> {
        let (chunk_pos, axial) = self.hex_to_chunk(&hex);

        if self.does_chunk_exist(chunk_pos) {
            let (q, r) = chunk_pos.sparse_index();
            let index = self.chunks_sparse[q][r].unwrap();
            return self.chunks[index].get_tile(&axial.to_hex());
        }
        None
    }
    
    pub fn get_tile_mut(&mut self, hex: Hex) -> Option<&mut T> {
        let (chunk_pos, axial) = self.hex_to_chunk(&hex);

        if self.does_chunk_exist(chunk_pos) {
            let (q, r) = chunk_pos.sparse_index();
            let index = self.chunks_sparse[q][r].unwrap();
            return self.chunks[index].get_tile_mut(&axial.to_hex());
        }
        None
    }
}

//
// Hex structs

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Hex {
    Axial(Axial),
    Cube(Cube),
}

impl std::ops::Add for Hex {
    type Output = Hex;

    fn add(self, other: Hex) -> Hex {
        match (self, other) {
            (Hex::Axial(a), Hex::Axial(b)) => (a + b).to_hex(),
            (Hex::Cube(a), Hex::Cube(b)) => (a + b).to_hex(),
            (Hex::Axial(axial), Hex::Cube(cube)) | (Hex::Cube(cube), Hex::Axial(axial)) => { 
                let cube = cube.to_axial(); 
                (axial + cube).to_hex()
            }, 
        }
    }
}

impl std::ops::AddAssign for Hex {
    fn add_assign(&mut self, other: Hex) {
        match (self, other) {
            (Hex::Axial(a), Hex::Axial(b)) => *a += b,
            (Hex::Cube(a), Hex::Cube(b)) => *a += b,
            (Hex::Axial(axial), Hex::Cube(cube)) => { 
                let cube = cube.to_axial(); 
                *axial += cube;
            },
            (Hex::Cube(cube), Hex::Axial(axial)) => {
                let axial = axial.to_cube();
                *cube += axial;
            }
        };
    }
}

impl std::ops::Add<Axial> for Hex {
    type Output = Hex;

    fn add(self, other: Axial) -> Hex {
        self + other.to_hex()
    }
}

impl std::ops::Add<Cube> for Hex {
    type Output = Hex;

    fn add(self, other: Cube) -> Hex {
        self + other.to_hex()
    }
}

impl std::ops::AddAssign<Axial> for Hex {
    fn add_assign(&mut self, other: Axial) {
        *self += other.to_hex();
    }
}

impl std::ops::AddAssign<Cube> for Hex {
    fn add_assign(&mut self, other: Cube) {
        *self += other.to_hex();
    }
}

impl Hex {
    pub fn to_axial(&self) -> Axial {
        match self {
            Hex::Axial(hex) => *hex,
            Hex::Cube(hex) => hex.to_axial(),
        }
    }

    pub fn neighbors(&self) -> [Hex; 6] {
        match self {
            Hex::Axial(Axial { q, r }) => {
                [
                    Hex::Axial(Axial::new(*q, *r-1)),
                    Hex::Axial(Axial::new(*q+1, *r-1)),
                    Hex::Axial(Axial::new(*q+1, *r)),
                    Hex::Axial(Axial::new(*q, *r+1)),
                    Hex::Axial(Axial::new(*q-1, *r+1)),
                    Hex::Axial(Axial::new(*q-1, *r)),
                ]
            } 
            Hex::Cube(Cube { q, r, s }) => {
                [
                    Hex::Cube(Cube::new(*q, *r-1, *s+1)),
                    Hex::Cube(Cube::new(*q+1, *r-1, *s)),
                    Hex::Cube(Cube::new(*q+1, *r, *s-1)),
                    Hex::Cube(Cube::new(*q, *r+1, *s-1)),
                    Hex::Cube(Cube::new(*q-1, *r+1, *s)),
                    Hex::Cube(Cube::new(*q-1, *r, *s+1)),
                ]
            }
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Axial {
    pub q: i32,
    pub r: i32,
}

impl std::ops::Add for Axial {
    type Output = Axial;

    fn add(mut self, other: Axial) -> Axial {
        self.q += other.q;
        self.r += other.r;
        self
    }
}

impl std::ops::AddAssign for Axial {
    fn add_assign(&mut self, other: Axial) {
        self.q += other.q;
        self.r += other.r;
    }
}

impl Axial {
    pub fn new(q: i32, r: i32) -> Axial {
        Axial {
            q, r
        }
    }

    pub fn to_cube(&self) -> Cube {
        Cube::new(self.q, self.r, -self.q - self.r)
    }

    pub fn to_hex(&self) -> Hex {
        Hex::Axial(*self)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Cube {
    pub q: i32,
    pub r: i32,
    pub s: i32,
}

impl std::ops::Add for Cube {
    type Output = Cube;

    fn add(self, other: Cube) -> Cube {
        (self.to_axial() + other.to_axial()).to_cube()
    }
}

impl std::ops::AddAssign for Cube {
    fn add_assign(&mut self, other: Cube) {
        let cube = (self.to_axial() + other.to_axial()).to_cube();

        self.q = cube.q;
        self.r = cube.r;
        self.s = cube.s;
    }
}

impl Cube {
    pub fn new(q: i32, r: i32, s: i32) -> Cube {
        Cube {
            q, r, s,
        }
    }

    pub fn to_axial(&self) -> Axial {
        debug_assert!(self.is_valid());

        Axial::new(self.q, self.r)
    }

    pub fn to_hex(&self) -> Hex {
        Hex::Cube(*self)
    }

    pub fn is_valid(&self) -> bool {
        self.q + self.r + self.s == 0
    }
}

#[derive(Copy, Clone, Debug)]
pub enum FractionalHex {
    FractionalAxial(FractionalAxial),
    FractionalCube(FractionalCube),
}

#[derive(Copy, Clone, Debug)]
pub struct FractionalAxial {
    pub q: f32,
    pub r: f32,
}

impl FractionalAxial {
    pub fn new(q: f32, r: f32) -> FractionalAxial {
        FractionalAxial {
            q, r
        }
    }

    pub fn to_fractional_cube(&self) -> FractionalCube {
        FractionalCube::new(self.q, self.r, -self.q - self.r)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct FractionalCube {
    pub q: f32,
    pub r: f32,
    pub s: f32,
}

impl FractionalCube {
    pub fn new(q: f32, r: f32, s: f32) -> FractionalCube {
        FractionalCube {
            q, r, s,
        }
    }

    pub fn to_fractional_axial(&self) -> FractionalAxial {
        debug_assert!(self.is_valid());

        FractionalAxial::new(self.q, self.r)
    } 

    pub fn to_cube(&self) -> Cube {
        let mut qi = self.q.round() as i32;
        let mut ri = self.r.round() as i32;
        let mut si = self.s.round() as i32;
    
        let q_diff = f64::abs(qi as f64 - self.q as f64);
        let r_diff = f64::abs(ri as f64 - self.r as f64);
        let s_diff = f64::abs(si as f64 - self.s as f64);
    
        if q_diff > r_diff && q_diff > s_diff {
            qi = -ri - si;
        } else if r_diff > s_diff {
            ri = -qi - si;
        } else {
            si = -qi - ri;
        }
    
        Cube::new(qi, ri, si)
    } 

    pub fn is_valid(&self) -> bool {
        f32::abs(self.q + self.r + self.s) < 0.05
    }
}
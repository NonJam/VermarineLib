use crate::tetra::math::Vec2;
use rand::SeedableRng;
use rand::Rng;
use rand::rngs::StdRng;

// Hex tile data

pub struct HexTileData {
    pub ground_height: u8,
    pub wall_height: u8,
}

impl HexTileData {
    pub fn new(height: u8) -> HexTileData {
        HexTileData {
            ground_height: height,
            wall_height: height,
        }
    }
}

// Hex map

pub struct HexMap {
    tiles: Vec<HexTileData>,
    pub width: usize,
    pub height: usize,
    pub position: Vec2<f32>,
    pub tallest: u8,

    pub hex_width: f32,
    pub hex_height: f32,
    pub hex_vert_step: f32,
    pub hex_depth_step: f32,

    pub wall_vert_step: f32,
    pub wall_vert_offset: f32,
}

impl HexMap {
    pub fn new(
        width: usize, 
        height: usize,             
        hex_width: f32,
        hex_height: f32,
        hex_vert_step: f32,
        hex_depth_step: f32,
        wall_vert_offset: f32,
        wall_vert_step: f32,
        max_hex_depth: u8,
    ) -> Self {
        let mut rand = StdRng::seed_from_u64(100);
        let mut tiles = Vec::<HexTileData>::with_capacity(width * height);
        let mut tallest = 0;
        for _ in 0..width * height {
            let value = rand.gen_range(0, max_hex_depth as u16 + 1) as u8;
            tiles.push(HexTileData::new(value));
            if value > tallest {
                tallest = value;
            }
        }

        let tile_space_bounds = Vec2::new(
            ((height - 1) as f32 / 2.) + width as f32,
            height as f32,
        );
        let mut world_space_bounds = tile_space_bounds * Vec2::new(hex_width, hex_vert_step);
        world_space_bounds.y += hex_height - hex_vert_step;        
        let position = world_space_bounds / -2.;

        HexMap {
            tiles,
            width,
            height,
            position,
            tallest,

            hex_width,
            hex_height,
            hex_vert_step,
            hex_depth_step,

            wall_vert_offset,
            wall_vert_step,
        }
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
        let size_y = 18.66666666666666666;

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

            let axial_hex = self.pixel_to_hex_raw(pos.clone(), height_offset);
            let cube_hex = axial_hex.to_fractional_cube();
            let axial_hex = cube_hex.to_cube().to_axial();
    
            if axial_hex.q < 0 || (axial_hex.q as usize) >= self.width || 
                axial_hex.r < 0 || (axial_hex.r as usize) >= self.height {
                continue;
            }

            let tile = self.get_tile(&Hex::Axial(axial_hex));

            if tile.wall_height != height {
                continue;
            }

            if tallest_height.is_none() || tile.wall_height > tallest_height.unwrap().0 {
                tallest_height = Some((tile.wall_height, axial_hex));
            }
        }

        if let Some((_, hex)) = tallest_height {
            return Some(hex);
        }
        None
    }

    pub fn axial_to_pixel(&mut self, hex: Axial) -> (f32, f32) {
        let size_x = self.hex_width / f32::sqrt(3.0);
        // this value is derived by solving for X in:
        // FLOOR_VERT_STEP * R = X * (3.0 / 2.0 * R) 
        // R can be 1 so we can simplify to:
        // FLOOR_VERT_STEP = X * 1.5
        // X = FLOOR_VERT_STEP / 1.5
        let size_y = 18.66666666666666666;

        let x = size_x * (f32::sqrt(3.0) * hex.q as f32 + f32::sqrt(3.0) / 2.0 * hex.r as f32);
        let y = size_y * (3.0 / 2.0 * hex.r as f32);
        (
            x + 18. + self.position.x,
            y + 18. + self.position.y,
        )
    }

    pub fn get_tile_at_index_mut(&mut self, index: usize) -> &mut HexTileData {
        &mut self.tiles[index]
    }

    pub fn get_tile_at_index(&self, index: usize) -> &HexTileData {
        &self.tiles[index]
    }

    pub fn get_tile(&self, hex: &Hex) -> &HexTileData {
        let hex = hex.as_axial();
        
        if hex.q < 0 || (hex.q as usize) >= self.width {
            panic!();
        }
        if hex.r < 0 || (hex.r as usize) >= self.height {
            panic!();
        }

        let q = hex.q as usize;
        let r = hex.r as usize;
        self.tiles.get(q + (r * self.width)).unwrap()
    }
}

//
// Hex structs

#[derive(Copy, Clone, Debug)]
pub enum Hex {
    Axial(Axial),
    Cube(Cube),
}

impl Hex {
    pub fn as_axial(&self) -> Axial {
        match self {
            Hex::Axial(hex) => *hex,
            Hex::Cube(hex) => hex.to_axial(),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Axial {
    pub q: i32,
    pub r: i32,
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
}

#[derive(Copy, Clone, Debug)]
pub struct Cube {
    pub q: i32,
    pub r: i32,
    pub s: i32,
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
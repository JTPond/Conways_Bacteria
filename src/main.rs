use std::error;
use std::fs::File;
use std::convert::TryInto;

extern crate rayon;
use rayon::prelude::*;

extern crate rand;
use rand::prelude::*;

extern crate gif;
use gif::{Encoder, Frame};

extern crate calcify;
use calcify::{Tree, Collection, Bin, Point};
use calcify::io::ToFile;

const BOARD_SIZE: usize = 2000;
const SEED_PROB: f64 = 0.5;
const TIME_STEPS: usize = 300;

#[derive(Debug, PartialEq, Copy, Clone)]
struct Bacteria {
    pub i: u16,
    pub j: u16,
    pub height: u16,
    neighbors: [u16; 8],
}

impl Bacteria {
    pub fn new(i: u16, j: u16) -> Bacteria {
        let mut rng = rand::thread_rng();
        let y: f64 = rng.gen();
        let height = if y < SEED_PROB { 1 } else { 0 };
        Bacteria {
            i,
            j,
            height,
            neighbors: [0; 8],
        }
    }

    pub fn init_neighbors(&mut self, lab: &[Bacteria]) {
        let l_index: usize = BOARD_SIZE - 1;
        let jj: usize = self.j.into();
        let ii: usize = self.i.into();
        self.neighbors = [
            if jj > 0 { lab[(jj-1)*BOARD_SIZE + ii].height } else { 0 },
            if jj > 0 && ii < l_index { lab[(jj-1)*BOARD_SIZE + ii + 1].height } else { 0 },
            if ii < l_index { lab[(jj)*BOARD_SIZE + ii + 1].height } else { 0 },
            if jj < l_index && ii < l_index { lab[(jj+1)*BOARD_SIZE + ii + 1].height } else { 0 },
            if jj < l_index { lab[(jj+1)*BOARD_SIZE + ii].height } else { 0 },
            if jj < l_index && ii > 0 { lab[(jj+1)*BOARD_SIZE + ii - 1].height } else { 0 },
            if ii > 0 { lab[(jj)*BOARD_SIZE + ii -1].height } else { 0 },
            if jj > 0 && ii > 0 { lab[(jj-1)*BOARD_SIZE + ii - 1].height } else { 0 },
        ];
    }

    fn single_tick(&mut self) {
        let t_sum: u16 = self.neighbors.iter().sum();
        if t_sum == 3 {
            if self.height == 0 {
                self.height = 1;
            }
        }
        else if t_sum < 2 || t_sum > 3{
            if t_sum == 4 {
                let e_sum: u16 = self.neighbors.iter().enumerate().filter(|(i,_)| i%2 == 0).map(|(_,x)| x).sum();
                let o_sum: u16 = self.neighbors.iter().enumerate().filter(|(i,_)| i%2 != 0).map(|(_,x)| x).sum();
                if e_sum == 4 || o_sum == 4 {
                    self.height += 1;
                } else {
                    if self.height == 1 {
                        self.height = 0;
                    }
                }
            } else {
                if self.height == 1 {
                    self.height = 0;
                }
            }
        }
    }

    fn colony_tick(&mut self, n_max: u16) {
        let t_sum: u16 = self.neighbors.iter().sum::<u16>();
        if t_sum >= (8*(self.height-1)).into() {
            if self.height > 0 {
                self.height += 1;
            }
        }
        else if self.height == 0 {
            if t_sum >= 4*n_max {
                self.height += 1;
            }
        }
    }

    pub fn tick(&mut self) {
        if let Some(n_max) = self.neighbors.iter().max() {
            let b_max = *n_max;
            if b_max <= 1 && self.height <= 1 {
                self.single_tick();
            } else if b_max < 3 && self.height <= 3  {
                self.colony_tick(b_max);
            }
        }
    }
}

fn main() -> Result<(),Box<dyn error::Error>> {
    let mut frame: Vec<Bacteria> = [0;BOARD_SIZE*BOARD_SIZE].iter().enumerate().map(|(x,_)|{
        let ii: u16 = (x % BOARD_SIZE).try_into().unwrap();
        let jj: u16 = (x / BOARD_SIZE).try_into().unwrap();
        Bacteria::new(ii,jj)
    }).collect();
    let mut n_frame: Vec<Bacteria>;

    let mut image = File::create(format!("./scratch/seed_test_{}.gif",SEED_PROB)).unwrap();
    // let mut pixels: Vec<u8> = Vec::new();
    // for cp in (0..=255).into_iter().rev() {
    //     for _ in 0..3 {
    //         pixels.push(cp);
    //     }
    // }
    // let mut encoder = Encoder::new(&mut image, BOARD_SIZE.try_into().unwrap(),
    //                                             BOARD_SIZE.try_into().unwrap(),
    //                                             &pixels).unwrap();
    let mut encoder = Encoder::new(&mut image, BOARD_SIZE.try_into().unwrap(),
                                               BOARD_SIZE.try_into().unwrap(), &[255,255,255,
                                                                                 215,225,215, //D7E1D7
                                                                                 108,135,109, //#6C876D
                                                                                 28,61,30, //#1C3D1E
                                                                                 3,26,4 //#031A04
                                                                                ]).unwrap();

    let mut max_points: Collection<Point> = Collection::empty();
    let mut maxes: Collection<f64> = Collection::empty();
    let mut tots: Collection<f64> = Collection::empty();

    tots.push(
        frame.iter().map(|b| b.height as f64).sum::<f64>()
    );
    maxes.push(1.0);

    let pixels: Vec<u8> = frame.iter().map(|b| b.height as u8).collect();
    encoder.write_frame(&Frame::from_indexed_pixels(BOARD_SIZE.try_into().unwrap(),
                                                    BOARD_SIZE.try_into().unwrap(),
                                                    &pixels,
                                                    None)).unwrap();

    for _t in 0..TIME_STEPS {
        n_frame = frame.par_iter().map(|b|{
            let mut ib = *b;
            ib.init_neighbors(&frame[..]);
            ib
        }).collect();
        frame = n_frame.par_iter().map(|b|{
            let mut ib = *b;
            ib.tick();
            ib
        }).collect();
        tots.push(
            frame.iter().map(|b| b.height as f64).sum::<f64>()
        );
        if let Some(n_max) = frame.iter().map(|b| b.height).max() {
            if n_max > 3 {
                for (i,bb) in frame.iter().filter(|b| b.height == n_max).enumerate() {
                    if n_frame[i].height == n_max - 1 {
                        max_points.push(Point::new(bb.i as f64, bb.j as f64));
                    }
                }
            }
            maxes.push(n_max as f64);
        }
        let pixels: Vec<u8> = frame.iter().map(|b| b.height as u8).collect();
        let mut g_frame = Frame::from_indexed_pixels(BOARD_SIZE.try_into().unwrap(),
                                                        BOARD_SIZE.try_into().unwrap(),
                                                        &pixels,
                                                        None);
        g_frame.delay = 16;
        encoder.write_frame(&g_frame).unwrap();
    }
    let max_dist: Collection<Bin> = maxes.hist(5);
    let mut ttree = Tree::new("Bacteria Data");
    ttree.add_field("Desc","Data for a test run of Conways Bacteria")?;
    ttree.add_field("Details",&format!("BOARD_SIZE: {}, SEED_PROB: {}, TIME_STEPS: {}",BOARD_SIZE,SEED_PROB,TIME_STEPS))?;
    ttree.add_branch("Max Heights",maxes,"f64")?;
    ttree.add_branch("Max Points",max_points,"Point")?;
    ttree.add_branch("Total Heights",tots,"f64")?;
    ttree.add_branch("Height dists",max_dist,"Bin")?;
    ttree.write_msg(&format!("./scratch/seed_test_{}.msg",SEED_PROB))?;
    Ok(())
}

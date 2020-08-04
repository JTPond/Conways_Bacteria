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
const SEED_PROB: f64 = 0.005;

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
        if t_sum > 1 && t_sum < 4{
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

    fn colony_tick(&mut self) {
        let t_sum: u16 = self.neighbors.iter().sum::<u16>();
        if t_sum > (8*self.height).into() {
            if self.height > 0 {
                self.height += 1;
            }
        } else if t_sum == (4*self.height).into() {
            if self.height == 0 {
                self.height += 1;
            }
        }
    }

    pub fn tick(&mut self) {
        if let Some(n_max) = self.neighbors.iter().max() {
            if n_max <= &1 && self.height <= 1 {
                self.single_tick();
            } else if n_max < &4 && self.height <= 4  {
                self.colony_tick();
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

    let mut image = File::create("./scratch/test_2.gif").unwrap();
    let mut encoder = Encoder::new(&mut image, BOARD_SIZE.try_into().unwrap(),
                                               BOARD_SIZE.try_into().unwrap(), &[255,255,255,
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

    for _t in 0..100 {
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
            if n_max >= 2 {
                for bb in frame.iter().filter(|b| b.height == n_max) {
                    max_points.push(Point::new(bb.i as f64, bb.j as f64));
                }
            }
            maxes.push(n_max as f64);
        }
        let pixels: Vec<u8> = frame.iter().map(|b| b.height as u8).collect();
        let mut g_frame = Frame::from_indexed_pixels(BOARD_SIZE.try_into().unwrap(),
                                                        BOARD_SIZE.try_into().unwrap(),
                                                        &pixels,
                                                        None);
        g_frame.delay = 33;
        encoder.write_frame(&g_frame).unwrap();
    }
    let max_dist: Collection<Bin> = maxes.hist(10);
    let mut ttree = Tree::new("Bacteria Data");
    ttree.add_branch("Max Heights",maxes,"f64")?;
    ttree.add_branch("Max Points",max_points,"Point")?;
    ttree.add_branch("Total Heights",tots,"f64")?;
    ttree.add_branch("Height dists",max_dist,"Bin")?;
    ttree.write_msg("./scratch/test_2.msg")?;
    Ok(())
}

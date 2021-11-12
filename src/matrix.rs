use rand::distributions::uniform::SampleRange;
use rand::distributions::Standard;
use rand::Rng;

use crate::strain::{Alive, LimitedCounter, Strain};

/// Contains information a whole area of matrix.
///
/// A matrix is generic over the elements of type `T` it stores.
pub struct Matrix<T> {
    height: u16,
    width: u16,
    strains: Vec<Option<Strain<Alive>>>,
    strain_spawn_chance: f64,
    strain_start_interval: std::ops::Range<u16>,
    strain_len_interval: std::ops::Range<u16>,
    strain_speed_interval: std::ops::Range<u16>,
    rng: Box<dyn rand::RngCore>,
    /// 2D map cells cache. Updated internally after every tick.
    pub map: Vec<Vec<T>>,
}

impl<T> std::fmt::Display for Matrix<T>
where
    T: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for i in 0..self.height {
            for j in 0..self.width {
                write!(f, "{}", self.map[i as usize][j as usize])?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl<T> std::fmt::Debug for Matrix<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("State")
            .field("strains", &self.strains)
            .field("strain_spawn_chance", &self.strain_spawn_chance)
            .field("strain_start_interval", &self.strain_start_interval)
            .field("strain_len_interval", &self.strain_len_interval)
            .field("strain_speed_interval", &self.strain_speed_interval)
            .field("map", &self.map)
            .finish()
    }
}

impl<T> Matrix<T>
where
    T: Default + Clone,
{
    /// Creates a new state with the specified parameters.
    ///
    /// ## Panics
    ///
    /// Can panic if the supplied `strain_len_interval` includes 0. In other
    /// words, the length of strain must always be greater than 0.
    pub fn new(
        height: u16,
        width: u16,
        strain_spawn_chance: f64,
        strain_len_interval: std::ops::Range<u16>,
        strain_speed_interval: std::ops::Range<u16>,
        rng: Box<dyn rand::RngCore>,
    ) -> Self {
        Matrix {
            height,
            width,
            strains: (0..width)
                .map(|_| None)
                .collect::<Vec<Option<Strain<Alive>>>>(),
            strain_spawn_chance,
            strain_start_interval: 0..1,
            strain_len_interval,
            strain_speed_interval,
            rng,
            map: vec![vec![T::default(); width as usize]; height as usize],
        }
    }

    /// Checks and updates the matrix size.
    ///
    /// This method will do nothing if the size is unchanged.
    pub fn notify_size(&mut self, new_width: u16, new_height: u16) {
        let mut updated = false;
        if self.height != new_height {
            self.height = new_height;
            updated = true;
        }
        match self.width {
            w if w > new_width => {
                self.strains.truncate(new_width as usize);
                self.width = new_width;
                updated = true;
            }
            w if w < new_width => {
                self.strains
                    .extend((0..new_width - self.width).map(|_| None));
                self.width = new_width;
                updated = true;
            }
            _ => (),
        }
        if updated {
            self.map = vec![vec![T::default(); new_width as usize]; new_height as usize];
        }
    }

    /// Ticks the matrix.
    pub fn tick<F>(&mut self, f: F)
    where
        F: FnMut(Option<u16>) -> T,
    {
        self.tick_strains();
        self.generate_strains();
        self.update_raw_map(f);
    }

    /// Ticks the existing matrix strains.
    fn tick_strains(&mut self) {
        self.strains
            .iter_mut()
            .filter(|x| x.is_some())
            .for_each(|x| {
                *x = match x.take().unwrap().tick(self.height) {
                    Ok(s) => Some(s),
                    Err(_) => {
                        let r: f64 = self.rng.sample(Standard);
                        if r <= self.strain_spawn_chance {
                            Some(Strain::new(
                                self.strain_start_interval
                                    .clone()
                                    .sample_single(&mut self.rng),
                                self.strain_len_interval
                                    .clone()
                                    .sample_single(&mut self.rng),
                                LimitedCounter::new(
                                    self.strain_speed_interval
                                        .clone()
                                        .sample_single(&mut self.rng),
                                ),
                            ))
                        } else {
                            None
                        }
                    }
                }
            });
    }

    fn generate_strains(&mut self) {
        self.strains
            .iter_mut()
            .filter(|x| x.is_none())
            .for_each(|x| {
                let r: f64 = self.rng.sample(Standard);
                if r <= self.strain_spawn_chance {
                    *x = Some(Strain::new(
                        self.strain_start_interval
                            .clone()
                            .sample_single(&mut self.rng),
                        self.strain_len_interval
                            .clone()
                            .sample_single(&mut self.rng),
                        LimitedCounter::new(
                            self.strain_speed_interval
                                .clone()
                                .sample_single(&mut self.rng),
                        ),
                    ));
                }
            })
    }

    /// Updates the raw map of the state.
    ///
    /// This map will be used to update the terminal buffer.
    fn update_raw_map<F>(&mut self, f: F)
    where
        F: FnMut(Option<u16>) -> T,
    {
        let mut f = f;
        (0..self.width as usize).for_each(|i| {
            let mut strain_buf = if let Some(strain) = self.strains[i].as_ref() {
                strain.mask(self.height, &mut f)
            } else {
                self.empty_mask(&mut f)
            }
            .into_iter();
            (0..self.height as usize).for_each(|j| {
                self.map[j][i] = strain_buf.next().unwrap();
            });
        })
    }

    //// Generates an empty strain mask for the current matrix.
    fn empty_mask<F>(&self, f: F) -> Vec<T>
    where
        F: FnMut(Option<u16>) -> T,
    {
        let mut f = f;
        (0..self.height).map(|_| f(None)).collect::<Vec<T>>()
    }
}

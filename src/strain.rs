use std::marker::PhantomData;

#[derive(Debug)]
pub struct Alive;

#[derive(Debug)]
pub struct Dead;

#[derive(Debug)]
pub struct Strain<T> {
    head: u16,
    len: u16,
    orig_len: u16,
    start: u16,
    speed: LimitedCounter,
    _p: std::marker::PhantomData<fn(T) -> T>,
}

#[derive(Debug, Clone, Copy)]
pub struct LimitedCounter {
    count: u16,
    limit: u16,
}

impl LimitedCounter {
    pub fn new(limit: u16) -> Self {
        LimitedCounter { count: 0, limit }
    }

    fn tick(&mut self) -> bool {
        self.count = (self.count + 1) % self.limit;
        self.count == 0
    }
}

impl<Alive> Strain<Alive> {
    pub(crate) fn new(start: u16, len: u16, speed: LimitedCounter) -> Strain<Alive> {
        assert!(len > 0, "len must be greater than 0");
        Self {
            head: start,
            len,
            orig_len: len,
            start,
            speed,
            _p: PhantomData::default(),
        }
    }

    pub(crate) fn tick(mut self, bound: u16) -> Result<Strain<Alive>, Strain<Dead>> {
        if self.start >= bound {
            return Err(self.kill());
        }
        if self.speed.tick() {
            self.head += 1;
            if self.head >= bound {
                let eaten = self.head - bound + 1;
                self.head -= eaten;
                if self.len <= eaten {
                    return Err(self.kill());
                }
                self.len -= eaten;
            }
        }
        Ok(self)
    }

    /// Creates the strain mask.
    pub(crate) fn mask<T, F>(&self, bound: u16, f: F) -> Vec<T>
    where
        F: FnMut(Option<u16>) -> T,
    {
        let mut f = f;
        let mut mask = Vec::with_capacity(bound as usize);
        if self.start >= bound {
            (0..bound).for_each(|_| mask.push(f(None)));
            return mask;
        }
        (0..self.start).for_each(|_| mask.push(f(None)));
        //let bound = bound - self.start;
        if (self.head - self.start) <= self.len - 1 {
            (0..=(self.head - self.start)).for_each(|i| {
                mask.push(f(Some(
                    (self.head - self.start) - i + (self.orig_len - self.len),
                )))
            });
            (0..mask.capacity() - mask.len()).for_each(|_| mask.push(f(None)));
        } else {
            (0..(self.head - self.start) - (self.len - 1)).for_each(|_| mask.push(f(None)));
            (0..self.len)
                .for_each(|i| mask.push(f(Some(self.len - i - 1 + (self.orig_len - self.len)))));
            (0..mask.capacity() - mask.len()).for_each(|_| mask.push(f(None)));
        }
        mask
    }

    fn kill(self) -> Strain<Dead> {
        Strain {
            head: self.head,
            len: self.len,
            orig_len: self.orig_len,
            start: self.start,
            speed: self.speed,
            _p: PhantomData::default(),
        }
    }

    // This function can be greatly improved.
    pub(crate) fn bump_bound(self, bound: u16) -> Result<Strain<Alive>, Strain<Dead>> {
        let mut cloned = self.clone();
        Ok(cloned.tick(bound).map(|_| self)?)
    }
}

impl<Alive> Clone for Strain<Alive> {
    fn clone(&self) -> Self {
        Self {
            head: self.head.clone(),
            len: self.len.clone(),
            orig_len: self.orig_len.clone(),
            start: self.start.clone(),
            speed: self.speed.clone(),
            _p: self._p.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strain_creation() {
        let strain: Strain<Alive> = Strain::new(0, 3, LimitedCounter::new(1));
        let bound = 5;
        let expected = vec![1, 0, 0, 0, 0];
        let mask = strain.mask(bound, |x| match x {
            Some(_) => 1,
            None => 0,
        });
        assert_eq!(expected, mask);
    }

    #[test]
    fn small_strain_cycle() {
        let mut strain: Strain<Alive> = Strain::new(0, 3, LimitedCounter::new(1));
        let bound = 5;
        {
            macro_rules! cycle_advance {
                ($expected:expr) => {
                    assert_eq!(
                        $expected,
                        strain.mask(bound, |x| match x {
                            Some(_) => 1,
                            None => 0,
                        })
                    );
                };
                ($expected:expr;) => {
                    cycle_advance!($expected);
                    strain = strain.tick(bound).unwrap();
                };
            }

            cycle_advance!(vec![1, 0, 0, 0, 0];);
            cycle_advance!(vec![1, 1, 0, 0, 0];);
            cycle_advance!(vec![1, 1, 1, 0, 0];);
            cycle_advance!(vec![0, 1, 1, 1, 0];);
            cycle_advance!(vec![0, 0, 1, 1, 1];);
            cycle_advance!(vec![0, 0, 0, 1, 1];);
            cycle_advance!(vec![0, 0, 0, 0, 1]);

            assert!(strain.tick(bound).is_err());
        }
    }

    #[test]
    fn long_strain_cycle() {
        let mut strain: Strain<Alive> = Strain::new(0, 6, LimitedCounter::new(1));
        let bound = 5;
        {
            macro_rules! cycle_advance {
                ($expected:expr) => {
                    assert_eq!(
                        $expected,
                        strain.mask(bound, |x| match x {
                            Some(_) => 1,
                            None => 0,
                        })
                    );
                };
                ($expected:expr;) => {
                    cycle_advance!($expected);
                    strain = strain.tick(bound).unwrap();
                };
            }

            cycle_advance!(vec![1, 0, 0, 0, 0];);
            cycle_advance!(vec![1, 1, 0, 0, 0];);
            cycle_advance!(vec![1, 1, 1, 0, 0];);
            cycle_advance!(vec![1, 1, 1, 1, 0];);
            cycle_advance!(vec![1, 1, 1, 1, 1];);
            cycle_advance!(vec![1, 1, 1, 1, 1];);
            cycle_advance!(vec![0, 1, 1, 1, 1];);
            cycle_advance!(vec![0, 0, 1, 1, 1];);
            cycle_advance!(vec![0, 0, 0, 1, 1];);
            cycle_advance!(vec![0, 0, 0, 0, 1]);

            assert!(strain.tick(bound).is_err());
        }
    }

    #[test]
    fn custom_start_strain_cycle() {
        let mut strain: Strain<Alive> = Strain::new(3, 3, LimitedCounter::new(1));
        let bound = 5;
        {
            macro_rules! cycle_advance {
                ($expected:expr) => {
                    assert_eq!(
                        $expected,
                        strain.mask(bound, |x| match x {
                            Some(_) => 1,
                            None => 0,
                        })
                    );
                };
                ($expected:expr;) => {
                    cycle_advance!($expected);
                    strain = strain.tick(bound).unwrap();
                };
            }

            cycle_advance!(vec![0, 0, 0, 1, 0];);
            cycle_advance!(vec![0, 0, 0, 1, 1];);
            cycle_advance!(vec![0, 0, 0, 1, 1];);
            cycle_advance!(vec![0, 0, 0, 0, 1]);

            assert!(strain.tick(bound).is_err());
        }
    }
}

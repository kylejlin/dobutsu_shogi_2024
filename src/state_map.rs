use super::*;

#[derive(Clone, Debug)]
pub struct StateMap<T> {
    raw: [Option<Box<Bucket0<T>>>; 256 * 256],
}

pub type StateMapNode<T> = [Option<Box<T>>; 16];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bucket5<T>(pub [T; 16]);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bucket4<T>(pub StateMapNode<Bucket5<T>>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bucket3<T>(pub StateMapNode<Bucket4<T>>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bucket2<T>(pub StateMapNode<Bucket3<T>>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bucket1<T>(pub StateMapNode<Bucket2<T>>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bucket0<T>(pub StateMapNode<Bucket1<T>>);

#[derive(Clone, Copy, Debug)]
pub struct DidAddendAlreadyExist {
    pub did_addend_already_exist: bool,
}

pub trait Null: Sized + PartialEq + Eq {
    fn null() -> Self;

    fn is_null(&self) -> bool {
        *self == Self::null()
    }
}

impl<T: Null + Copy> Null for Bucket5<T> {
    fn null() -> Self {
        Self([Null::null(); 16])
    }
}

impl<T: Null + Copy> Null for Bucket4<T> {
    fn null() -> Self {
        Self(core::array::from_fn(|_| Null::null()))
    }
}

impl<T: Null + Copy> Null for Bucket3<T> {
    fn null() -> Self {
        Self(core::array::from_fn(|_| Null::null()))
    }
}

impl<T: Null + Copy> Null for Bucket2<T> {
    fn null() -> Self {
        Self(core::array::from_fn(|_| Null::null()))
    }
}

impl<T: Null + Copy> Null for Bucket1<T> {
    fn null() -> Self {
        Self(core::array::from_fn(|_| Null::null()))
    }
}

impl<T: Null + Copy> Null for Bucket0<T> {
    fn null() -> Self {
        Self(core::array::from_fn(|_| Null::null()))
    }
}

impl<T: Eq> Null for Option<T> {
    fn null() -> Self {
        None
    }
}

impl<T: Null> Null for Box<T> {
    fn null() -> Self {
        Box::new(Null::null())
    }
}

impl Null for State {
    fn null() -> Self {
        Self(0)
    }
}

impl Null for StateAndStats {
    fn null() -> Self {
        Self(0)
    }
}

impl Null for StateStats {
    fn null() -> Self {
        // `!0` can never represent a valid `StateStats` value,
        // because it would imply the 127 required child reports,
        // which is impossible
        // (since 8 pieces * 12 destination squares = 96,
        // which gives a conservative upper bound on the number
        // of possible actions, and thus, the number of possible children).
        Self(!0)
    }
}

impl<T: Copy + Null + std::fmt::Debug> StateMap<T> {
    pub fn empty() -> Self {
        Self {
            raw: core::array::from_fn(|_| None),
        }
    }

    pub fn add(&mut self, state: State, value: T) -> DidAddendAlreadyExist {
        let bucket0 = self.raw[(state.0 >> (40 - 16)) as usize].get_or_insert_with(Null::null);
        let bucket1 = bucket0.0[((state.0 >> (40 - 16 - 4)) & 0b1111) as usize]
            .get_or_insert_with(Null::null);
        let bucket2 = bucket1.0[((state.0 >> (40 - 16 - 2 * 4)) & 0b1111) as usize]
            .get_or_insert_with(Null::null);
        let bucket3 = bucket2.0[((state.0 >> (40 - 16 - 3 * 4)) & 0b1111) as usize]
            .get_or_insert_with(Null::null);
        let bucket4 = bucket3.0[((state.0 >> (40 - 16 - 4 * 4)) & 0b1111) as usize]
            .get_or_insert_with(Null::null);
        let bucket5 = &mut bucket4.0[((state.0 >> (40 - 16 - 5 * 4)) & 0b1111) as usize]
            .get_or_insert_with(Null::null);
        let item = &mut bucket5.0[((state.0 >> (40 - 16 - 6 * 4)) & 0b1111) as usize];

        let did_addend_already_exist = !item.is_null();

        *item = value;

        DidAddendAlreadyExist {
            did_addend_already_exist,
        }
    }

    pub fn get(&self, state: State) -> T {
        let Some(bucket0) = self.raw[(state.0 >> (40 - 16)) as usize].as_ref() else {
            return Null::null();
        };

        let Some(bucket1) = bucket0.0[((state.0 >> (40 - 16 - 4)) & 0b1111) as usize].as_ref()
        else {
            return Null::null();
        };

        let Some(bucket2) = bucket1.0[((state.0 >> (40 - 16 - 2 * 4)) & 0b1111) as usize].as_ref()
        else {
            return Null::null();
        };

        let Some(bucket3) = bucket2.0[((state.0 >> (40 - 16 - 3 * 4)) & 0b1111) as usize].as_ref()
        else {
            return Null::null();
        };

        let Some(bucket4) = bucket3.0[((state.0 >> (40 - 16 - 4 * 4)) & 0b1111) as usize].as_ref()
        else {
            return Null::null();
        };

        let Some(bucket5) = bucket4.0[((state.0 >> (40 - 16 - 5 * 4)) & 0b1111) as usize].as_ref()
        else {
            return Null::null();
        };

        bucket5.0[((state.0 >> (40 - 16 - 6 * 4)) & 0b1111) as usize]
    }

    pub fn get_mut(&mut self, state: State) -> Option<&mut T> {
        let Some(bucket0) = self.raw[(state.0 >> (40 - 16)) as usize].as_mut() else {
            return None;
        };

        let Some(bucket1) = bucket0.0[((state.0 >> (40 - 16 - 4)) & 0b1111) as usize].as_mut()
        else {
            return None;
        };

        let Some(bucket2) = bucket1.0[((state.0 >> (40 - 16 - 2 * 4)) & 0b1111) as usize].as_mut()
        else {
            return None;
        };

        let Some(bucket3) = bucket2.0[((state.0 >> (40 - 16 - 3 * 4)) & 0b1111) as usize].as_mut()
        else {
            return None;
        };

        let Some(bucket4) = bucket3.0[((state.0 >> (40 - 16 - 4 * 4)) & 0b1111) as usize].as_mut()
        else {
            return None;
        };

        let Some(bucket5) = bucket4.0[((state.0 >> (40 - 16 - 5 * 4)) & 0b1111) as usize].as_mut()
        else {
            return None;
        };

        let out = &mut bucket5.0[((state.0 >> (40 - 16 - 6 * 4)) & 0b1111) as usize];

        if out.is_null() {
            return None;
        }

        Some(out)
    }

    pub fn union(mut self, other: &Self) -> Self {
        other.visit_in_key_order(|state, value| {
            self.add(state, value);
        });

        self
    }

    pub fn to_sorted_vec(&self) -> Vec<(State, T)> {
        let mut raw = Vec::new();

        self.visit_in_key_order(|state, value| raw.push((state, value)));

        raw
    }

    /// This will visit the entries in the order of their keys
    /// (defined by `<State as Ord>::cmp`).
    pub fn visit_in_key_order(&self, mut visitor: impl FnMut(State, T)) {
        for (i0, bucket0) in self.raw.iter().enumerate() {
            let Some(bucket0) = bucket0 else {
                continue;
            };
            let prefix = (i0 as u64) << (40 - 16);
            self.visit0(prefix, bucket0, &mut visitor);
        }
    }

    fn visit0(&self, prefix: u64, bucket0: &Bucket0<T>, mut visitor: impl FnMut(State, T)) {
        for (i1, bucket1) in bucket0.0.iter().enumerate() {
            let Some(bucket1) = bucket1 else {
                continue;
            };
            let prefix = prefix | ((i1 as u64) << (40 - 16 - 4));
            self.visit1(prefix, bucket1, &mut visitor);
        }
    }

    fn visit1(&self, prefix: u64, bucket1: &Bucket1<T>, mut visitor: impl FnMut(State, T)) {
        for (i2, bucket2) in bucket1.0.iter().enumerate() {
            let Some(bucket2) = bucket2 else {
                continue;
            };
            let prefix = prefix | ((i2 as u64) << (40 - 16 - 2 * 4));
            self.visit2(prefix, bucket2, &mut visitor);
        }
    }

    fn visit2(&self, prefix: u64, bucket2: &Bucket2<T>, mut visitor: impl FnMut(State, T)) {
        for (i3, bucket3) in bucket2.0.iter().enumerate() {
            let Some(bucket3) = bucket3 else {
                continue;
            };
            let prefix = prefix | ((i3 as u64) << (40 - 16 - 3 * 4));
            self.visit3(prefix, bucket3, &mut visitor);
        }
    }

    fn visit3(&self, prefix: u64, bucket3: &Bucket3<T>, mut visitor: impl FnMut(State, T)) {
        for (i4, bucket4) in bucket3.0.iter().enumerate() {
            let Some(bucket4) = bucket4 else {
                continue;
            };
            let prefix = prefix | ((i4 as u64) << (40 - 16 - 4 * 4));
            self.visit4(prefix, bucket4, &mut visitor);
        }
    }

    fn visit4(&self, prefix: u64, bucket4: &Bucket4<T>, mut visitor: impl FnMut(State, T)) {
        for (i5, bucket5) in bucket4.0.iter().enumerate() {
            let Some(bucket5) = bucket5 else {
                continue;
            };
            let prefix = prefix | ((i5 as u64) << (40 - 16 - 5 * 4));
            self.visit5(prefix, bucket5, &mut visitor);
        }
    }

    fn visit5(&self, prefix: u64, bucket5: &Bucket5<T>, mut visitor: impl FnMut(State, T)) {
        for i6 in 0..16 {
            let item = bucket5.0[i6];
            if !item.is_null() {
                let prefix = prefix | ((i6 as u64) << (40 - 16 - 6 * 4));
                visitor(State(prefix), item);
            }
        }
    }
}

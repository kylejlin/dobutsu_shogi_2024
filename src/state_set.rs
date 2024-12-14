use super::*;

#[derive(Clone, Debug)]
pub struct StateSet {
    raw: [Option<Box<Bucket0>>; 256 * 256],
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Bitset16(pub u16);

pub type StateSetNode<T> = [Option<Box<T>>; 16];

pub type Bucket5 = Bitset16;
pub type Bucket4 = [Bucket5; 16];
pub type Bucket3 = StateSetNode<Bucket4>;
pub type Bucket2 = StateSetNode<Bucket3>;
pub type Bucket1 = StateSetNode<Bucket2>;
pub type Bucket0 = StateSetNode<Bucket1>;

#[derive(Clone, Copy, Debug)]
pub struct DidAddendAlreadyExist {
    pub did_addend_already_exist: bool,
}

impl StateSet {
    pub fn empty() -> Self {
        let empty: Option<Box<Bucket0>> = Default::default();

        let mut v = Vec::with_capacity(256 * 256);

        for _ in 0..256 * 256 {
            v.push(empty.clone());
        }

        Self {
            raw: v.try_into().unwrap(),
        }
    }

    pub fn add(&mut self, state: State) -> DidAddendAlreadyExist {
        let bucket0 =
            self.raw[(state.0 >> (40 - 16)) as usize].get_or_insert_with(Default::default);
        let bucket1 = bucket0[((state.0 >> (40 - 16 - 4)) & 0b1111) as usize]
            .get_or_insert_with(Default::default);
        let bucket2 = bucket1[((state.0 >> (40 - 16 - 2 * 4)) & 0b1111) as usize]
            .get_or_insert_with(Default::default);
        let bucket3 = bucket2[((state.0 >> (40 - 16 - 3 * 4)) & 0b1111) as usize]
            .get_or_insert_with(Default::default);
        let bucket4 = bucket3[((state.0 >> (40 - 16 - 4 * 4)) & 0b1111) as usize]
            .get_or_insert_with(Default::default);
        let bucket5 = &mut bucket4[((state.0 >> (40 - 16 - 5 * 4)) & 0b1111) as usize];

        let i6 = (state.0 >> (40 - 16 - 6 * 4)) & 0b1111;
        let mask = 1 << i6;

        let did_addend_already_exist = bucket5.0 & mask != 0;

        bucket5.0 |= mask;

        DidAddendAlreadyExist {
            did_addend_already_exist,
        }
    }

    pub fn union(mut self, other: &Self) -> Self {
        other.visit_in_order(|state| {
            self.add(state);
        });

        self
    }

    pub fn to_sorted_vec(&self) -> Vec<State> {
        let mut raw = Vec::new();

        self.visit_in_order(|state| raw.push(state));

        raw
    }

    pub fn visit_in_order(&self, mut visitor: impl FnMut(State)) {
        for (i0, bucket0) in self.raw.iter().enumerate() {
            let Some(bucket0) = bucket0 else {
                continue;
            };
            let prefix = (i0 as u64) << (40 - 16);
            self.visit0(prefix, bucket0, &mut visitor);
        }
    }

    fn visit0(&self, prefix: u64, bucket0: &Bucket0, mut visitor: impl FnMut(State)) {
        for (i1, bucket1) in bucket0.iter().enumerate() {
            let Some(bucket1) = bucket1 else {
                continue;
            };
            let prefix = prefix | ((i1 as u64) << (40 - 16 - 4));
            self.visit1(prefix, bucket1, &mut visitor);
        }
    }

    fn visit1(&self, prefix: u64, bucket1: &Bucket1, mut visitor: impl FnMut(State)) {
        for (i2, bucket2) in bucket1.iter().enumerate() {
            let Some(bucket2) = bucket2 else {
                continue;
            };
            let prefix = prefix | ((i2 as u64) << (40 - 16 - 2 * 4));
            self.visit2(prefix, bucket2, &mut visitor);
        }
    }

    fn visit2(&self, prefix: u64, bucket2: &Bucket2, mut visitor: impl FnMut(State)) {
        for (i3, bucket3) in bucket2.iter().enumerate() {
            let Some(bucket3) = bucket3 else {
                continue;
            };
            let prefix = prefix | ((i3 as u64) << (40 - 16 - 3 * 4));
            self.visit3(prefix, bucket3, &mut visitor);
        }
    }

    fn visit3(&self, prefix: u64, bucket3: &Bucket3, mut visitor: impl FnMut(State)) {
        for (i4, bucket4) in bucket3.iter().enumerate() {
            let Some(bucket4) = bucket4 else {
                continue;
            };
            let prefix = prefix | ((i4 as u64) << (40 - 16 - 4 * 4));
            self.visit4(prefix, bucket4, &mut visitor);
        }
    }

    fn visit4(&self, prefix: u64, bucket4: &Bucket4, mut visitor: impl FnMut(State)) {
        for (i5, bucket5) in bucket4.iter().enumerate() {
            let prefix = prefix | ((i5 as u64) << (40 - 16 - 5 * 4));
            self.visit5(prefix, *bucket5, &mut visitor);
        }
    }

    fn visit5(&self, prefix: u64, bucket5: Bucket5, mut visitor: impl FnMut(State)) {
        for i6 in 0..16 {
            if bucket5.0 & (1 << i6) != 0 {
                let prefix = prefix | ((i6 as u64) << (40 - 16 - 6 * 4));
                visitor(State(prefix));
            }
        }
    }
}

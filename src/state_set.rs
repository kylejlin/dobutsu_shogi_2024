use super::*;

#[derive(Clone, Debug)]
pub struct StateSet {
    raw: [Option<Box<Bucket0>>; 256 * 256],
}

type StateSetNode<T> = [Option<Box<T>>; 16];

#[derive(Clone, Copy, Debug, Default)]
struct Bitset16(u16);

type Bucket5 = Bitset16;
type Bucket4 = [Bucket5; 16];
type Bucket3 = StateSetNode<Bucket4>;
type Bucket2 = StateSetNode<Bucket3>;
type Bucket1 = StateSetNode<Bucket2>;
type Bucket0 = StateSetNode<Bucket1>;

#[derive(Clone, Copy, Debug)]
pub struct DidAddendAlreadyExist {
    pub did_addend_already_exist: bool,
}

impl StateSet {
    pub fn empty() -> Self {
        let empty: Option<
            Box<StateSetNode<StateSetNode<StateSetNode<StateSetNode<[Bitset16; 16]>>>>>,
        > = Default::default();

        let mut v = Vec::with_capacity(256 * 256);

        for _ in 0..256 * 256 {
            v.push(empty.clone());
        }

        Self {
            raw: v.try_into().unwrap(),
        }
    }

    pub fn add(&mut self, node: SearchNode) -> DidAddendAlreadyExist {
        let bucket0 = self.raw[(node.0 >> (56 - 16)) as usize].get_or_insert_with(Default::default);
        let bucket1 = bucket0[((node.0 >> (56 - 16 - 4)) & 0b1111) as usize]
            .get_or_insert_with(Default::default);
        let bucket2 = bucket1[((node.0 >> (56 - 16 - 2 * 4)) & 0b1111) as usize]
            .get_or_insert_with(Default::default);
        let bucket3 = bucket2[((node.0 >> (56 - 16 - 3 * 4)) & 0b1111) as usize]
            .get_or_insert_with(Default::default);
        let bucket4 = bucket3[((node.0 >> (56 - 16 - 4 * 4)) & 0b1111) as usize]
            .get_or_insert_with(Default::default);
        let bucket5 = &mut bucket4[((node.0 >> (56 - 16 - 5 * 4)) & 0b1111) as usize];

        let i6 = (node.0 >> (56 - 16 - 6 * 4)) & 0b1111;
        let mask = 1 << i6;

        let did_addend_already_exist = bucket5.0 & mask != 0;

        bucket5.0 |= mask;

        DidAddendAlreadyExist {
            did_addend_already_exist,
        }
    }

    pub fn into_sorted_vec(self) -> Vec<SearchNode> {
        let mut raw = Vec::new();

        self.write(&mut raw);

        raw.sort_unstable();

        raw
    }

    fn write(&self, out: &mut Vec<SearchNode>) {
        for (i0, bucket0) in self.raw.iter().enumerate() {
            let Some(bucket0) = bucket0 else {
                continue;
            };
            let prefix = (i0 as u64) << (56 - 16);
            self.write0(prefix, bucket0, out);
        }
    }

    fn write0(&self, prefix: u64, bucket0: &Bucket0, out: &mut Vec<SearchNode>) {
        for (i1, bucket1) in bucket0.iter().enumerate() {
            let Some(bucket1) = bucket1 else {
                continue;
            };
            let prefix = prefix | ((i1 as u64) << (56 - 16 - 4));
            self.write1(prefix, bucket1, out);
        }
    }

    fn write1(&self, prefix: u64, bucket1: &Bucket1, out: &mut Vec<SearchNode>) {
        for (i2, bucket2) in bucket1.iter().enumerate() {
            let Some(bucket2) = bucket2 else {
                continue;
            };
            let prefix = prefix | ((i2 as u64) << (56 - 16 - 2 * 4));
            self.write2(prefix, bucket2, out);
        }
    }

    fn write2(&self, prefix: u64, bucket2: &Bucket2, out: &mut Vec<SearchNode>) {
        for (i3, bucket3) in bucket2.iter().enumerate() {
            let Some(bucket3) = bucket3 else {
                continue;
            };
            let prefix = prefix | ((i3 as u64) << (56 - 16 - 3 * 4));
            self.write3(prefix, bucket3, out);
        }
    }

    fn write3(&self, prefix: u64, bucket3: &Bucket3, out: &mut Vec<SearchNode>) {
        for (i4, bucket4) in bucket3.iter().enumerate() {
            let Some(bucket4) = bucket4 else {
                continue;
            };
            let prefix = prefix | ((i4 as u64) << (56 - 16 - 4 * 4));
            self.write4(prefix, bucket4, out);
        }
    }

    fn write4(&self, prefix: u64, bucket4: &Bucket4, out: &mut Vec<SearchNode>) {
        for (i5, bucket5) in bucket4.iter().enumerate() {
            let prefix = prefix | ((i5 as u64) << (56 - 16 - 5 * 4));
            self.write5(prefix, *bucket5, out);
        }
    }

    fn write5(&self, prefix: u64, bucket5: Bucket5, out: &mut Vec<SearchNode>) {
        for i6 in 0..16 {
            if bucket5.0 & (1 << i6) != 0 {
                let prefix = prefix | ((i6 as u64) << (56 - 16 - 6 * 4));
                out.push(SearchNode(prefix));
            }
        }
    }
}

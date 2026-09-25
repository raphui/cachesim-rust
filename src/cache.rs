use crate::types::Access;
use crate::types::AccessOperation;
use crate::pseudorandom_policy::PseudorandomPolicy;

const BYTES_PER_WORD: u32 = 1;
const MAX_BITS: u32 = 32;

#[derive(Clone, Copy, Default)]
pub struct CacheLine {
    pub tag: u32,
    pub valid: bool,
    pub block: u8,
}

struct CacheDesc {
    line_size: u32,
    nb_lines: u32,
    associativity: u32,
    nb_sets: u32,
    word_per_line: u32,
    offset_bit: u32,
    index_bit: u32,
    tag_bit: u32,
    offset_mask: u32,
    index_mask: u32,
    tag_mask: u32,
}

pub struct Cache {
    cache_desc: CacheDesc,
    replace_policy: PseudorandomPolicy,
    cache: Vec<CacheLine>,
    misses: u32,
    write_misses: u32,
    read_misses: u32,
    hits: u32,
    write_hits: u32,
    read_hits: u32,
    evictions: u32,
}

impl Cache {
    pub fn new(line_size: u32, nb_lines: u32, associativity: u32) -> Self {
        let nb_sets = nb_lines / associativity;
        let offset_bit = (line_size / BYTES_PER_WORD).ilog2();
        let index_bit = nb_sets.ilog2();
        let tag_bit = MAX_BITS - index_bit - offset_bit;

        let word_per_line = line_size / BYTES_PER_WORD;

        let offset_mask = (1 << offset_bit) - 1;
        let index_mask = ((1 << index_bit) - 1) << offset_bit;
        let tag_mask = ((1 << (MAX_BITS - (offset_bit + index_bit))) - 1) << (offset_bit + index_bit);

        let cache = vec![CacheLine::default(); nb_lines as usize];

        Cache {
            cache_desc: CacheDesc {
                line_size: line_size,
                nb_lines: nb_lines,
                associativity: associativity,
                nb_sets: nb_sets,
                word_per_line: word_per_line,
                offset_bit: offset_bit,
                index_bit: index_bit,
                tag_bit: tag_bit,
                offset_mask: offset_mask,
                index_mask: index_mask,
                tag_mask: tag_mask,
            },
            replace_policy: PseudorandomPolicy::new(),
            cache,
            misses: 0,
            write_misses: 0,
            read_misses: 0,
            hits: 0,
            write_hits: 0,
            read_hits: 0,
            evictions: 0,
        }
    }

    fn get_offset(&self, access: &Access) -> u32 {
        access.address & self.cache_desc.offset_mask
    }

    fn get_index(&self, access: &Access) -> u32 {
        (access.address & self.cache_desc.index_mask) >> self.cache_desc.offset_bit
    }

    fn get_tag(&self, access: &Access) -> u32 {
        (access.address & self.cache_desc.tag_mask) >>
            (self.cache_desc.offset_bit + self.cache_desc.index_bit)
    }

    fn start_of_set(&self, access: &Access) -> usize {
        self.get_index(access) as usize * self.cache_desc.associativity as usize
    }

    fn get_set(&self, access: &Access) -> &[CacheLine] {
        let index = self.get_index(&access) as usize;
        let start_index = index * self.cache_desc.associativity as usize;
        let end_index = start_index + self.cache_desc.associativity as usize;

        return &self.cache[start_index..end_index]
    }

    fn find_line(&self, access: &Access) -> Option<usize> {
        let tag = self.get_tag(&access);

        self.get_set(access)
            .iter()
            .position(|line| line.tag == tag && line.valid)
            .map(|way| self.start_of_set(access) + way)
    }

    fn find_free_line(&self, access: &Access) -> Option<usize> {
        self.get_set(access)
            .iter()
            .position(|line| !line.valid)
            .map(|way| self.start_of_set(access) + way)
    }

    fn is_set_full(&self, access: &Access) -> bool {
        let set = self.get_set(access);

        set.iter().all(|line| line.valid)
    }

    fn perform(&mut self, access: Access) {
        let line = self.find_line(&access);

        match line {
            None => {
                let idx: usize;
                let tag = self.get_tag(&access);

                if self.is_set_full(&access) {
                    idx = self.start_of_set(&access) +
                        self.replace_policy.find_evicted(self.cache_desc.associativity) as usize;

                    self.evictions += 1;
                } else {
                    /* unwrap should be safe because
                     * we checked that the set is not full just before
                     */
                    idx = self.find_free_line(&access).unwrap();
                }

                self.replace_policy.execute(&mut self.cache[idx], tag);

                if access.operation == AccessOperation::READ {
                    self.read_misses += 1;
                } else if access.operation == AccessOperation::WRITE {
                    self.write_misses += 1;
                }

                self.misses += 1;
            },
            Some(_) => {
                if access.operation == AccessOperation::READ {
                    self.read_hits += 1;
                } else if access.operation == AccessOperation::WRITE {
                    self.write_hits += 1;
                }

                self.hits += 1;
            }

        }

        self.replace_policy.post_op(self.cache_desc.associativity, &access);
    }

    pub fn read(&mut self, access: Access) {
        self.perform(access);
    }

    pub fn write(&mut self, access: Access) {
        self.perform(access);
    }
}

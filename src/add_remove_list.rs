use rand::distributions::{Distribution, Uniform};
use rand::prelude::*;
use rand::rngs::SmallRng;
use std::collections::HashMap;

use crate::add_remove;
use crate::add_remove::AddRemoveHow;

const CN_FOR_INV: u8 = 12;
// const E_RATIO: f64 = 0.20; 400K
const E_RATIO_BARR: f64 = 0.1300000;

#[derive(Clone, Debug)]
pub struct AddOrRemove {
    atom_to_position: HashMap<u32, usize, ahash::RandomState>,
    pub atoms: Vec<AtomPosChange>, // [(from, to, energy_change)]
    pub total_k: f64,
    // potential: f64,
}

#[derive(Clone, Debug)]
pub struct AtomPosChange {
    pos: u32,
    k: f64,
    how: AddRemoveHow,
}

impl AddOrRemove {
    pub fn new() -> AddOrRemove {
        let item_to_position: HashMap<u32, usize, ahash::RandomState> = HashMap::default();
        AddOrRemove {
            atom_to_position: item_to_position,
            atoms: Vec::new(),
            total_k: 0.,
        }
    }

    pub fn calc_total_cn_change(&mut self) {
        self.total_k = self.atoms.iter().map(|x| x.k).sum::<f64>();
    }

    pub fn cond_add_item(
        &mut self,
        pos: u32,
        cn: u8,
        atom_type: u8,
        temperature: f64,
        how: &add_remove::AddRemoveHow,
    ) {
        if cn >= 11 {
            return;
        }
        match how {
            add_remove::AddRemoveHow::Remove(remove_atom_type)
            | add_remove::AddRemoveHow::Exchange(remove_atom_type, _) => {
                if atom_type == *remove_atom_type {
                    match self.atom_to_position.entry(pos) {
                        std::collections::hash_map::Entry::Vacant(e) => {
                            let k = tst_rate_calculation(cn as f64 * E_RATIO_BARR, temperature);
                            self.atoms.push(AtomPosChange {
                                pos,
                                k,
                                how: how.clone(),
                            });
                            e.insert(self.atoms.len() - 1);
                            self.total_k += k;
                        }
                        _ => return,
                    }
                }
            }
            AddRemoveHow::Add(_) => {
                if atom_type == 255 {
                    match self.atom_to_position.entry(pos) {
                        std::collections::hash_map::Entry::Vacant(e) => {
                            let k = if cn == 0 {
                                tst_rate_calculation((100) as f64 * E_RATIO_BARR, temperature)
                            } else {
                                tst_rate_calculation((12 / cn) as f64 * E_RATIO_BARR, temperature)
                            };
                            self.atoms.push(AtomPosChange {
                                pos,
                                k,
                                how: how.clone(),
                            });
                            e.insert(self.atoms.len() - 1);
                            self.total_k += k;
                        }
                        _ => return,
                    }
                }
            }
            AddRemoveHow::RemoveAndAdd(_, _) => todo!(),
        }
    }

    pub fn remove_item(&mut self, pos: u32) {
        if let Some(position) = self.atom_to_position.remove(&pos) {
            let atom = self.atoms.pop().unwrap();
            if position != self.atoms.len() {
                let old_move = std::mem::replace(&mut self.atoms[position], atom);
                self.atom_to_position
                    .insert(self.atoms[position].pos, position);
                self.total_k -= old_move.k;
            } else {
                self.total_k -= atom.k;
            }
        }
    }

    pub fn cond_update_cn(&mut self, pos: u32, cn: u8, temperature: f64) {
        if cn == 0 || cn == 3 {
            self.remove_item(pos);
        }
        if let Some(position) = self.atom_to_position.get(&pos) {
            let k = tst_rate_calculation((12 / cn) as f64 * E_RATIO_BARR, temperature);
            self.total_k -= self.atoms[*position].k;
            self.atoms[*position].k = k;
            self.total_k += k;
        }
    }

    pub fn choose_ramdom_atom_to_remove(
        &self,
        rng_choose: &mut SmallRng,
    ) -> Option<(u32, f64, f64)> {
        let between = Uniform::new_inclusive(0., self.total_k);
        let mut k_time_rng = between.sample(rng_choose);
        // println!(
        //     "ktot: {} krng: {}",
        //     format!("{:e}", self.total_k),
        //     format!("{:e}", k_time_rng),
        // );
        let mut cur_cn = 0.;
        let mut res: Option<(u32, u32, f64, f64, f64, f64)> = None;

        for atom in self.atoms.iter() {
            cur_cn += atom.k;
            // println!("{}", cur_cn);
            if k_time_rng <= cur_cn {
                return Some((atom.pos, atom.k, self.total_k));
            }
        }
        return None;
    }

    pub fn iter(&self) -> std::slice::Iter<'_, AtomPosChange> {
        self.atoms.iter()
    }

    pub fn _len(&self) -> usize {
        self.atoms.len()
    }
}

fn tst_rate_calculation(e_barr: f64, temperature: f64) -> f64 {
    // let e_use = if e_diff.is_negative() { 0. } else { e_diff };
    // println!("barr {}", e_barr);
    // println!("diff {}", e_diff);
    const KB_joul: f64 = 1.380649e-23;
    const h_joul: f64 = 6.62607015e-34;
    const KB_DIV_H: f64 = KB_joul / h_joul;
    const KB_eV: f64 = 8.6173324e-5;
    (KB_joul * temperature / h_joul) * ((-e_barr) / (KB_eV * temperature)).exp()
}
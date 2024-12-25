use crate::error::Error;
use crate::utils::{musize, ures};
use array_init::from_iter;
use num_integer::Integer;
use std::iter;
use std::sync::LazyLock;

pub fn factorial(n: musize) -> Option<ures> {
    static CACHE: LazyLock<[ures; 21]> = LazyLock::new(|| {
        from_iter(
            iter::once(1)
                .chain(Factorial {
                    current: 1,
                    current_fac: 1,
                })
                .take(21),
        )
        .unwrap()
    });

    CACHE.get(n).copied()
}

#[allow(dead_code)]
pub fn choose(m: musize, n: musize) -> Result<Option<ures>, Error> {
    match n {
        0 => Ok(Some(1)),
        1 => Ok(Some(m as ures)),
        n if n > (m - n) => choose(m, m - n),
        n if n > m => Err(Error::InvalidState(
            format!("in valid argument of [{} choose {}]", m, n).into(),
        )),
        n => Ok(choose_inner(m, n)),
    }
}

fn choose_inner(m: musize, n: musize) -> Option<ures> {
    match n {
        0 => Some(1),
        1 => Some(m as ures),
        n if n > (m - n) => choose_inner(m, m - n),
        n => {
            let mut val1: ures = 1;
            let mut val2: ures = if n <= 20 {
                factorial(n)?
            } else {
                factorial(20)?
            };
            let m = m as ures;
            let n = n as ures;
            for i in 1..=n {
                let mut mul1 = m - n + i;
                let gcd = val2.gcd(&mul1);
                val2 /= gcd;
                mul1 /= gcd;

                if i > 20 {
                    let mut mul2 = i;
                    let gcd = val1.gcd(&mul2);
                    val1 /= gcd;
                    mul2 /= gcd;

                    let gcd = mul1.gcd(&mul2);
                    mul1 /= gcd;
                    mul2 /= gcd;

                    val1 = val1.checked_mul(mul1)?;
                    val2 = val2.checked_mul(mul2)?;
                } else {
                    val1 = val1.checked_mul(mul1)?;
                }
            }

            Some(val1)
        }
    }
}

struct Factorial {
    current: ures,
    current_fac: ures,
}

impl Iterator for Factorial {
    type Item = ures;

    fn next(&mut self) -> Option<Self::Item> {
        self.current_fac = self.current_fac.checked_mul(self.current)?;
        self.current = self.current.checked_add(1)?;
        Some(self.current_fac)
    }
}

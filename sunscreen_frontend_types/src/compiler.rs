use crate::params::{determine_params, PlainModulusConstraint};
use crate::{Context, Error, Params, Result, SchemeType, SecurityLevel};
use sunscreen_circuit::{Circuit};

#[derive(Debug, Clone)]
enum ParamsMode {
    Search,
    Manual(Params)
}

pub struct Compiler<F> 
where F: Fn(&Params) -> Context
{
    circuit: F,
    params_mode: ParamsMode,
    plain_modulus_constraint: Option<PlainModulusConstraint>,
    security_level: SecurityLevel,
    noise_margin: u32,
}

impl <F> Compiler<F> 
where F: Fn(&Params) -> Context
{
    pub fn with_circuit(circuit: F) -> Self 
    {
        Self {
            circuit,
            params_mode: ParamsMode::Search,
            plain_modulus_constraint: None,
            security_level: SecurityLevel::TC128,
            noise_margin: 10
        }
    }

    pub fn find_params(mut self) -> Self {
        self.params_mode = ParamsMode::Search;
        self
    }

    pub fn plain_modulus_constraint(mut self, p: PlainModulusConstraint) -> Self {
        self.plain_modulus_constraint = Some(p);
        self
    }

    pub fn with_params(mut self, params: &Params) -> Self {
        self.params_mode = ParamsMode::Manual(params.clone());
        self
    }

    pub fn security_level(mut self, security_level: SecurityLevel) -> Self {
        self.security_level = security_level;
        self
    }

    pub fn noise_margin_bits(mut self, noise_margin: u32) -> Self {
        self.noise_margin = noise_margin;
        self
    }

    pub fn compile(self) -> Result<(Circuit, Params)> {
        let (circuit, params) = match self.params_mode {
            ParamsMode::Manual(p) => {
                ((self.circuit)(&p), p.clone())
            },
            ParamsMode::Search => {
                let constraint = self.plain_modulus_constraint.ok_or(Error::MissingPlainModulusConstraint)?;

                let params = determine_params(&self.circuit, constraint, self.security_level, self.noise_margin, SchemeType::Bfv)?;

                ((self.circuit)(&params), params.clone())
            }
        };

        Ok((circuit.compile(), params))
    }
}
//!Utils
///Utils

use bs58;
use std::convert::TryInto;
use solana_program::{
    msg,
};

///Utilities
pub struct Utils {}

impl Utils {
    ///Get data from records
    pub fn get_data(data: &str, name: &str)->Result<[u8;32],bs58::decode::Error>{
        let records = data.clone();
        let idx= &records.find(&name);
        if !idx.is_some(){
            return Err(bs58::decode::Error::BufferTooSmall);
        }
        let empty = "";
        let sep : Vec<_> = records.match_indices("|").map(|(i, _)|i).collect();
        for i in &sep {
            if i > &idx.unwrap(){
                 let key = &records[(idx.unwrap()+name.len())..*i];
                 msg!(name);
                 let decoded = bs58::decode(&key).into_vec();
                 match decoded{
                    Ok(val)=>{
                        let code: &[u8] = &val;
                        let code_32 = code.try_into().expect("slice with incorrect length");
                        return Ok(code_32);
                    },
                    Err(error)=>{
                        return Err(error);
                    }
                }
            }
        }
        return Err(bs58::decode::Error::BufferTooSmall);
    }

}
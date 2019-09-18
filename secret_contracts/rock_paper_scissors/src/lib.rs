// Built-In Attributes
#![no_std]
#![allow(unused_attributes)] // https://github.com/rust-lang/rust/issues/60050

// Imports
extern crate eng_wasm;
extern crate eng_wasm_derive;
extern crate rustc_hex;
extern crate serde;
extern crate multihash;
extern crate enigma_crypto;

use eng_wasm::*;
use eng_wasm_derive::pub_interface;
use eng_wasm_derive::eth_contract;
use eng_wasm::String;
use serde::{Serialize, Deserialize};
use enigma_crypto::{KeyPair, hash::Keccak256};
use rustc_hex::ToHex;
use multihash::{encode, Hash};




// RockPaperScissors contract abi
#[eth_contract("./RockPaperScissors.json")]
struct EthContract;

// State key name "rock_paper_scissors_addr" holding eth address of RockPaperScissors contract
static ROCK_PAPER_SCISSORS_ADDR: &str = "rock_paper_scissors_addr";

// State key name "games" holding a vector of Game structs
static GAMES: &str = "games";

// Struct representing a game
#[derive(Serialize, Deserialize)]
pub struct Game {
    status: u8,
    player_1: H160,
    move_1: String,
    player_2: H160,
    move_2: String,
    winner: H160
}

// Public-facing secret contract function declarations
#[pub_interface]
pub trait ContractInterface{
    fn construct(rock_paper_scissors_addr: H160);
    fn new_game(player_1: H160, move_1: String);
    fn join_game(game_id: U256, pub_key: H256, move_2: String, sig: Vec<u8>);
}

pub struct Contract;

// Private functions accessible only by the secret contract
impl Contract {
    // Read voting address of RockPaperScissors contract
    fn get_rock_paper_scissors_addr() -> String {
        read_state!(ROCK_PAPER_SCISSORS_ADDR).unwrap_or_default()
    }

    // Read secret contract state to obtain vector of Games (or new vector if uninitialized)
    fn get_games() -> Vec<Game> {
        read_state!(GAMES).unwrap_or_default()
    }

    //fn prepare_hash_multiple<B: AsRef<[u8]>>(messages: &[B]) -> Vec<u8> {
        // wasmi is using a 32 bit target as oppose to the actual machine that
        // is a 64 bit target. therefore using u64 and not usize
    //    let mut res = Vec::with_capacity(messages.len() * mem::size_of::<u64>());
    //    for msg in messages {
    //        let msg = msg.as_ref();
    //        let len = (msg.len() as u64).to_be_bytes();
    //        res.extend_from_slice(&len);
    //        res.extend_from_slice(&msg);
    //    }
    //    res
    //}

    /// verify if the address that is sending the tokens is the one who actually sent the transfer.
    fn verify(signer: H256, msg: Vec<u8>, sig: Vec<u8>) -> bool {
        //let msg = [&addr.to_vec()[..], &amount.as_u64().to_be_bytes()];
        //let to_verify = Self::prepare_hash_multiple(&msg);
        let mut new_sig: [u8; 65] = [0u8; 65];
        new_sig.copy_from_slice(&sig[..65]);

        //let accepted_pubkey = KeyPair::recover(&to_verify, new_sig).unwrap();
        let accepted_pubkey = KeyPair::recover(&msg, new_sig).unwrap();
        *signer == *accepted_pubkey.keccak256()
    }

    fn derive_address(pub_key: H256) -> H160 {
        let hash = encode(Hash::Keccak256, &pub_key.as_bytes()).unwrap();
        let address = H160::from_slice(&hash[11..]);
        return address;
    }
}

impl ContractInterface for Contract {
    // Constructor function that takes in VotingETH ethereum contract address
    #[no_mangle]
    fn construct(rock_paper_scissors_addr: H160) {
        let rock_paper_scissors_str: String = rock_paper_scissors_addr.to_hex();
        write_state!(ROCK_PAPER_SCISSORS_ADDR => rock_paper_scissors_str);
    }

    #[no_mangle]
    fn new_game(player_1: H160, move_1: String) {
        //Uncomment when testing is done:
        //let hash = encode(Hash::Keccak256, &move_1.as_bytes()).unwrap();

        //The following 3 lines stay comments:
        //let message = Message::parse_slice(hash);
        //let signature = Signature::parse_slice(sig.as_bytes());
        //let key = PublicKey::parse_slice(player_1.to_fixed_bytes());

        //Uncomment when testing is done:
        //assert!(Self::verify(pub_key, hash, sig));
        //let player_1 = Self::derive_address(pub_key);
        let mut games = Self::get_games();
        //let game_id = U256::from(games.len());
        games.push(Game {
            status: 1,
            player_1: player_1,
            move_1: move_1,
            player_2: H160([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]),
            move_2: String::from(""),
            winner: H160([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]),
        });
        write_state!(GAMES => games);
        //let rock_paper_scissors_addr: String = Self::get_rock_paper_scissors_addr();
        //let c = EthContract::new(&rock_paper_scissors_addr);
        //c.newGame(game_id, player_1);
    }

    #[no_mangle]
    fn join_game(game_id: U256, pub_key: H256, move_2: String, sig: Vec<u8>) {
        let hash = encode(Hash::Keccak256, &move_2.as_bytes()).unwrap();
        assert!(Self::verify(pub_key, hash, sig));
        let player_2 = Self::derive_address(pub_key);
        let mut games = Self::get_games();
        match games.get(game_id.as_usize()) {
            Some(game) => {
                let status = game.status;
                if status == 1 {
                    let player_1 = game.player_1.clone();
                    let move_1 = game.move_1.clone();
                    let mut winner = H160([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]);
                    if (move_1 == String::from("rock") && move_2 == String::from("scissors")) ||
                       (move_1 == String::from("paper") && move_2 == String::from("rock")) ||
                       (move_1 == String::from("scissors") && move_2 == String::from("paper")) {
                      winner = player_1.clone();
                    } else if (move_1 == String::from("rock") && move_2 == String::from("paper")) ||
                              (move_1 == String::from("paper") && move_2 == String::from("scissors")) ||
                              (move_1 == String::from("scissors") && move_2 == String::from("rock")) {
                      winner = player_2.clone();
                    }
                    games[game_id.as_usize()] = Game {
                        status: 2,
                        player_1: player_1,
                        move_1: move_1,
                        player_2: player_2,
                        move_2: move_2,
                        winner: winner
                    };
                    write_state!(GAMES => games);
                    let rock_paper_scissors_addr: String = Self::get_rock_paper_scissors_addr();
                    let c = EthContract::new(&rock_paper_scissors_addr);
                    c.setWinner(game_id, player_2, winner);
                }
            },
            None => return,
        }
    }
}

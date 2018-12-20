use bitcoin::OutPoint;
use std::path::Path;
use std::fs;
use bitcoin::util::hash::Sha256dHash;
use bitcoin::network::serialize::BitcoinHash;
use std::path::PathBuf;
use std::io::Write;
use std::io::Read;
use rgb::proof::Proof;

#[derive(Debug, Clone)]
pub struct Database {
    basedir: Box<PathBuf>
}

impl Database {
    pub fn new(basedir: &Path) -> Database {
        let db = Database {
            basedir: Box::new(basedir.to_owned())
        };

        db.init();

        db
    }

    fn init(&self) {
        if !self.basedir.as_path().exists() {
            fs::create_dir(self.basedir.as_path());
        }
    }

    pub fn get_proofs_for(&self, outpoint: &OutPoint) -> Vec<Proof> {
        use bitcoin::network::serialize::deserialize;

        let mut ans = Vec::new();

        let outpoint_str = outpoint.txid.be_hex_string() + ":" + outpoint.vout.to_string().as_str();

        let mut proof_path = self.basedir.clone();
        proof_path.push(outpoint_str);

        if !proof_path.as_path().exists() {
            return ans;
        }

        for entry in proof_path.as_path().read_dir().expect("read_dir call failed") {
            if let Ok(entry) = entry {
                let mut file = fs::File::open(entry.path()).unwrap();
                let mut buffer: Vec<u8> = Vec::new();

                file.read_to_end(&mut buffer);

                let decoded: Proof = deserialize(&mut buffer).unwrap();
                ans.push(decoded);
            }
        }

        ans
    }

    pub fn save_proof(&self, proof: &Proof, txid: &Sha256dHash) {
        for out in &proof.output {
            let outpoint_str = txid.be_hex_string() + ":" + out.get_vout().to_string().as_str();

            let mut proof_path = self.basedir.clone();
            proof_path.push(outpoint_str);
            proof_path.push(proof.bitcoin_hash().be_hex_string());

            if !proof_path.as_path().parent().unwrap().exists() {
                fs::create_dir_all(&proof_path.as_path().parent().unwrap());
            }

            use bitcoin::network::serialize::RawEncoder;
            use bitcoin::network::encodable::ConsensusEncodable;

            let mut encoded: Vec<u8> = Vec::new();
            let mut enc = RawEncoder::new(encoded);
            proof.consensus_encode(&mut enc);

            let mut doc = fs::File::create(proof_path.as_path()).unwrap();
            doc.write(&enc.into_inner());
        }
    }
}
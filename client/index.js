var solana_web3 = require('@solana/web3.js');
var spl_token = require("@solana/spl-token");

async function testVesting(connection, account){
    let programId = new solana_web3.PublicKey("HSAMPxyGhyM15hYp6DKHhPVdtqtKSkeAEp9eeY7ixEDb",);
    const target_account = "5gsmZASNC26PJQJCRs8tEAb2meNAUAd3TfmJ5TghV4RN"
    let seeds = '';
    for (let i = 0; i < 31; i++) {
      seeds += Math.floor(Math.random() * 10);
    }
    seeds = Buffer.from(seeds);

    const [vestingAccountKey, bump] = await solana_web3.PublicKey.findProgramAddress(
        [seeds],
        programId,
    );
      

    var key_dict = "system_program_account:"+solana_web3.SystemProgram.programId+"|rent_sysvar_account:"+solana_web3.SYSVAR_RENT_PUBKEY+"|payer_account:"+account.publicKey+"|vesting_account:"+vestingAccountKey+"|";
    var buf = Buffer.from(key_dict);

    
    seeds = Buffer.from(seeds.toString('hex') + bump.toString(16), 'hex');

    let buffers = [
        Buffer.from(Int8Array.from([0]).buffer),
        seeds,
        Buffer.from(Int8Array.from([5]).buffer),
        Buffer.alloc(3),
        buf,
        Buffer.alloc(32),
      ];

      const data = Buffer.concat(buffers);
      
      console.log(programId);
      const keys = [
        {
          pubkey: solana_web3.SystemProgram.programId, //system_program_account
          isSigner: false,
          isWritable: false,
        },
        {
          pubkey: solana_web3.SYSVAR_RENT_PUBKEY, //rent
          isSigner: false,
          isWritable: false,
        },
        {
          pubkey: account.publicKey, //payer 
          isSigner: true,
          isWritable: true,
        },
        {
          pubkey: vestingAccountKey, //vesting
          isSigner: false,
          isWritable: true,
        },
      ];

    
    const instruction = new solana_web3.TransactionInstruction({
        keys: keys,
        programId: programId,
        data: data,
    });
    console.log("account debug:", account.publicKey.toBase58())
    solana_web3.sendAndConfirmTransaction(
        connection,
        new solana_web3.Transaction().add(instruction),
        [account],
        {
            skipPreflight: true,
            commitment: "confirmed",
        },
    ).then(()=>{console.log("done")}).catch((e)=>{console.log("error",e)});
}

async function main() {
    connection = new solana_web3.Connection("https://api.devnet.solana.com", 'confirmed');
    const payer = solana_web3.Keypair.generate();
    const lamports = 1*1000000000
    connection.requestAirdrop(payer.publicKey, lamports).then(()=>{
        console.log("airdrop done")
        testVesting(connection, payer)
    });
}
main()
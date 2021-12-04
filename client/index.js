var solana_web3 = require('@solana/web3.js');
var spl_token = require("@solana/spl-token");

async function testVesting(connection, account){
    let programId = new solana_web3.PublicKey("3jwszvTDRQp8UakHTAd5EJ1gURqNbuWcRsGbtU31gNYL",);
    let seeds = '';
    for (let i = 0; i < 64; i++) {
      seeds += Math.floor(Math.random() * 10);
    }
    seeds = seeds.slice(0, 31);
    seeds = Buffer.from(seeds);

    const [vestingAccountKey, bump] = await solana_web3.PublicKey.findProgramAddress(
        [seeds],
        programId,
    );
      
    seeds = Buffer.from(seeds.toString('hex') + bump.toString(16), 'hex');
    let buffers = [
        Buffer.from(Int8Array.from([0]).buffer),
        seeds,
        Buffer.alloc(32),
      ];
      
      buffers = buffers.slice(0,31);
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
const fs = require('fs');
const path = require('path');
const dotenv = require('dotenv');
const RockPaperScissors = artifacts.require("RockPaperScissors");
const {Enigma, utils, eeConstants} = require('enigma-js/node');
//const eu = require('ethereumjs-util')
const EthereumTx = require('ethereumjs-tx').Transaction
const hdkey = require('ethereumjs-wallet/hdkey');
const wallet = hdkey.fromMasterSeed(process.env.SEED).getWallet();
const address = wallet.getChecksumAddressString();

var EnigmaContract;
if(typeof process.env.SGX_MODE === 'undefined' || (process.env.SGX_MODE != 'SW' && process.env.SGX_MODE != 'HW' )) {
    console.log(`Error reading ".env" file, aborting....`);
    process.exit();
} else if (process.env.SGX_MODE == 'SW') {
  EnigmaContract = require('../build/enigma_contracts/EnigmaSimulation.json');
} else {
  EnigmaContract = require('../build/enigma_contracts/Enigma.json');
}
const EnigmaTokenContract = require('../build/enigma_contracts/EnigmaToken.json');


function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

let enigma = null;
let contractAddr;

contract("RockPaperScissors", accounts => {
  before(function() {
    enigma = new Enigma(
      web3,
      EnigmaContract.networks['4447'].address,
      EnigmaTokenContract.networks['4447'].address,
      'http://localhost:3346',
      {
        gas: 4712388,
        gasPrice: 100000000000,
        from: accounts[0],
      },
    );
    enigma.admin();

    contractAddr = fs.readFileSync('./test/rock_paper_scissors.txt', 'utf-8');
  })

  let task;
  it('should execute new_game task', async () => {
    let taskFn = 'new_game(address,string)';
    let taskArgs = [
      [address, 'address'],
      ['rock', 'string']
    ];
    let taskGasLimit = 6000000;
    let taskGasPx = utils.toGrains(1);

    task = await new Promise((resolve, reject) => {
      enigma.computeTask(taskFn, taskArgs, taskGasLimit, taskGasPx, accounts[0], contractAddr)
        .on(eeConstants.SEND_TASK_INPUT_RESULT, (result) => resolve(result))
        .on(eeConstants.ERROR, (error) => reject(error));
    });
  });

  it('should get the pending task', async () => {
    console.log(task)
    task = await enigma.getTaskRecordStatus(task);
    expect(task.ethStatus).to.equal(1);
  });

  it('should get the confirmed task', async () => {
    do {
      await sleep(1000);
      task = await enigma.getTaskRecordStatus(task);
      process.stdout.write('Waiting. Current Task Status is '+task.ethStatus+'\r');
    } while (task.ethStatus != 2);
    expect(task.ethStatus).to.equal(2);
    process.stdout.write('Completed. Final Task Status is '+task.ethStatus+'\n');
  }, 10000);
/*
  it('should get the result and verify the computation is correct', async () => {
    task = await new Promise((resolve, reject) => {
      enigma.getTaskResult(task)
        .on(eeConstants.GET_TASK_RESULT_RESULT, (result) => resolve(result))
        .on(eeConstants.ERROR, (error) => reject(error));
    });
    expect(task.engStatus).to.equal('SUCCESS');
    task = await enigma.decryptTaskResult(task);
    expect(parseInt(task.decryptedOutput, 16)).to.equal(76+17);
  });
*/
})

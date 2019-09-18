pragma solidity 0.5.8;

import './SafeMath.sol';

contract RockPaperScissors {
  using SafeMath for uint256;

  address payable public owner;
  address public enigma;

  struct Game {
    uint8 status;
    uint256 start;
    address payable winner;
    address payable player1;
    address payable player2;
    uint256 purse;
    uint256 cut;
    uint256 bet1;
    uint256 bet2;
  }

  //mapping(bytes32 => uint8) public results;
  mapping(uint256 => Game) public games;
  mapping(address => int256) public earnings;
  uint256 ownerPercent;


  // Constructor called when new contract is deployed
  constructor(address _enigmaAddress, uint256 _ownerPercent) public {
    owner = msg.sender;
    enigma = _enigmaAddress;
    ownerPercent = _ownerPercent;
  }

  //Create new game, pass bet and encrypted move
  function newGame(uint256 gameID, address payable player1)
  onlyEnigma()
  external {
    games[gameID].status = 1;
    games[gameID].start = now;
    games[gameID].player1 = player1;
    emit NewGame(gameID, player1);
  }

  function placeBet(uint256 gameID)
  payable
  external {
    require(games[gameID].status == 1); //Cannot place bet on a game that hasn't started or already finished
    require(games[gameID].purse == 0); //Cannot place bet if the purse has already been set
    if(games[gameID].bet1 == 0){
      require(games[gameID].player1 == msg.sender);
      games[gameID].bet1 = msg.value;
      emit BetPlaced(gameID, msg.sender, msg.value, true);
    } else {
      games[gameID].player2 = msg.sender;
      if(games[gameID].bet1 > msg.value){
        //Get the original bet made by player 1
        uint256 bet = games[gameID].bet1;
        games[gameID].bet1 = msg.value;
        games[gameID].bet2 = msg.value;
        //Return funds leftover because player 2 made a lower bet than player 1
        games[gameID].player1.transfer(bet.sub(msg.value));
      } else if(games[gameID].bet1 <= msg.value){
        //player 2 matches player 1's bet. Additional funds will be returned to player 2
        games[gameID].bet2 = games[gameID].bet1;
        if(games[gameID].bet1 < msg.value){
          //Return funds leftover because player 2 made a higher bet than player 1
          msg.sender.transfer(msg.value.sub(games[gameID].bet1));
        }
      }
      uint256 total = games[gameID].bet1.add(games[gameID].bet2);
      games[gameID].cut = total.getFractionalAmount(ownerPercent);
      games[gameID].purse = total.sub(games[gameID].cut);
      emit BetPlaced(gameID, msg.sender, games[gameID].bet2, false);
      emit Purse(gameID, games[gameID].purse, games[gameID].cut);
    }
  }

  function setWinner(uint256 gameID, address player2, address payable winner) public onlyEnigma() {
    require(games[gameID].status == 1);
    if(games[gameID].player2 != player2 || games[gameID].purse == 0){
      //Bets refunded
      refund(gameID);
    }
    games[gameID].status = 2;
    games[gameID].winner = winner;
    emit Winner(gameID, winner);
  }

  //Withdraw winnings
  function withdraw(uint256 gameID)
  external{
    require(games[gameID].status == 2);
    games[gameID].status = 3;
    uint winnings;
    if(games[gameID].winner == address(0)){
      games[gameID].player1.transfer(games[gameID].bet1);
      emit Withdraw(gameID, games[gameID].player1, games[gameID].bet1);
      games[gameID].player1.transfer(games[gameID].bet2);
      emit Withdraw(gameID, games[gameID].player2, games[gameID].bet2);
    } else {
      earnings[games[gameID].player1] -= int256(games[gameID].bet1);
      earnings[games[gameID].player2] -= int256(games[gameID].bet2);
      earnings[games[gameID].winner] += int256(games[gameID].purse);
      owner.transfer(games[gameID].cut);
      games[gameID].winner.transfer(games[gameID].purse);
      emit Withdraw(gameID, games[gameID].winner, games[gameID].purse);
    }
  }

  function cancelBet(uint256 gameID)
  external{
    require(games[gameID].status == 1);
    require(games[gameID].start.add(604800) < now); //Bets can be cancelled if the game has not completed after a week
    refund(gameID);
  }

  function getGame(uint256 gameID)
  external
  view
  returns(uint8, address, address, uint256, uint256, uint256){
    return (games[gameID].status,
            games[gameID].player1,
            games[gameID].player2,
            games[gameID].purse,
            games[gameID].bet1,
            games[gameID].bet2);
  }

  function getWinner(uint256 gameID)
  external
  view
  returns(address){
    return games[gameID].winner;
  }

  function getStatus(uint256 gameID)
  external
  view
  returns(uint8){
    return games[gameID].status;
  }

  function getEarnings(address _address)
  external
  view
  returns(int256){
    return earnings[_address];
  }

  function refund(uint256 gameID)
  private
  returns(bool){
    if(games[gameID].bet1 > 0){
      uint256 bet1 = games[gameID].bet1;
      games[gameID].bet1 = 0;
      games[gameID].player1.transfer(bet1);
    }

    if(games[gameID].bet2 > 0){
      uint256 bet2 = games[gameID].bet2;
      games[gameID].bet2 = 0;
      games[gameID].player2.transfer(bet2);
    }
    games[gameID].purse = 0;
    games[gameID].cut = 0;
    emit Purse(gameID, 0, 0);
  }

  // Modifier to ensure only enigma contract can call function
  modifier onlyEnigma() {
    require(msg.sender == enigma);
    _;
  }

  // Event emitted upon callback completion; watched from front end
  event Winner(uint256 indexed gameID, address indexed winner);
  event NewGame(uint256 indexed gameID, address indexed player1);
  event Withdraw(uint256 indexed gameID, address indexed player, uint256 indexed amount);
  event BetPlaced(uint256 indexed gameID, address indexed player,  uint256 bet, bool firstBet);
  event Purse(uint256 indexed gameID, uint256 purse, uint256 cut);
}

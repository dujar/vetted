// SPDX-License-Identifier: MIT
// Week-1 spike replica (step 2 task 6) — disposable proof of the demo's
// upgrade beat, faithful to the calibrated genuine-token pattern:
// beacon proxy (EIP-1967 beacon slot) + global pause + per-address blocklist.
// DIVERGENCE (deliberate, recorded in findings.md): isBlocked(address) is
// externally DECODABLE here, while the genuine 4663 tokens revert on the same
// selector — the replica lets the Stylus probe measure the decodable path;
// step 6 decides which behavior the demo replicas carry.
pragma solidity ^0.8.28;

contract StockToken {
    // --- calibrated selector surface (matches live 4663 tokens) ---
    // paused()            0x5c975abb
    // isBlocked(address)  0xfbac3951
    // uid()               0xf514ce36

    bool public paused;
    mapping(address => bool) public isBlocked;
    string public uid;

    string public name;
    string public symbol;
    uint8 public constant decimals = 18;
    uint256 public totalSupply;
    address public issuer;
    mapping(address => uint256) public balanceOf;
    mapping(address => mapping(address => uint256)) public allowance;

    event Transfer(address indexed from, address indexed to, uint256 value);
    event Approval(address indexed owner, address indexed spender, uint256 value);

    modifier whenNotPaused() {
        require(!paused, "TOKEN_PAUSED");
        _;
    }

    function initialize(string calldata name_, string calldata symbol_, string calldata uid_) external {
        require(issuer == address(0), "already initialized");
        issuer = msg.sender;
        name = name_;
        symbol = symbol_;
        uid = uid_;
    }

    function pause() external { paused = true; }
    function unpause() external { paused = false; }
    function blockAddress(address a) external { isBlocked[a] = true; }
    function unblockAddress(address a) external { isBlocked[a] = false; }

    function transfer(address to, uint256 amount) external whenNotPaused returns (bool) {
        // hidden-modifier equivalent: blocklist enforced inside transfer()
        require(!isBlocked[msg.sender], "SENDER_BLOCKED");
        require(!isBlocked[to], "RECEIVER_BLOCKED");
        return _move(msg.sender, to, amount);
    }

    function transferFrom(address from, address to, uint256 amount) external whenNotPaused returns (bool) {
        require(!isBlocked[from], "SENDER_BLOCKED");
        uint256 a = allowance[from][msg.sender];
        require(a >= amount, "allowance");
        if (a != type(uint256).max) allowance[from][msg.sender] = a - amount;
        return _move(from, to, amount);
    }

    function approve(address spender, uint256 amount) external returns (bool) {
        allowance[msg.sender][spender] = amount;
        emit Approval(msg.sender, spender, amount);
        return true;
    }

    function mint(address to, uint256 amount) external {
        require(msg.sender == issuer, "issuer only");
        totalSupply += amount;
        balanceOf[to] += amount;
        emit Transfer(address(0), to, amount);
    }

    function _move(address from, address to, uint256 amount) internal returns (bool) {
        require(balanceOf[from] >= amount, "balance");
        balanceOf[from] -= amount;
        balanceOf[to] += amount;
        emit Transfer(from, to, amount);
        return true;
    }
}

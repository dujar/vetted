// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

/// @notice IMPOSTOR TWIN 1 — the "naive copy" of the leaked pattern (this is
/// ALSO the spike replica's shape, the documented anti-pattern): pause and
/// blocklist held IN THE TOKEN, no beacon, no proxy. Same name/symbol/decimals
/// as the replica plus a fabricated registry-style uid() for phishy realism —
/// everything a casual check would see. Fails FINGERPRINT.md F2/F3/F4
/// (no EIP-1967 beacon layout, no implementation()-answering forwarder, no
/// beacon to answer the blocklist probe); F5's name/symbol passes, which is
/// the trap. Demo artifact — demo/assets.md labels it; the scanner must
/// reject it (spec.md:80).
contract ImpostorPlain {
    string public name;
    string public symbol;
    uint8 public constant decimals = 18;
    uint256 public totalSupply;
    address public owner;

    bool public paused; // decoy: pause held in-token (wrong place)
    mapping(address => bool) public isBlocked; // decoy: blocklist held in-token (wrong place)
    mapping(address => uint256) public balanceOf;
    mapping(address => mapping(address => uint256)) public allowance;
    bytes32 public uid; // fabricated registry-style id — excluded from the fingerprint, not proof of anything

    event Transfer(address indexed from, address indexed to, uint256 value);
    event Approval(address indexed owner, address indexed spender, uint256 value);

    error OwnerOnly();
    error Paused();
    error Blocked();

    modifier onlyOwner() {
        if (msg.sender != owner) revert OwnerOnly();
        _;
    }

    constructor(string memory name_, string memory symbol_, bytes32 uid_) {
        name = name_;
        symbol = symbol_;
        uid = uid_;
        owner = msg.sender;
    }

    function mint(address to, uint256 amount) external onlyOwner {
        totalSupply += amount;
        balanceOf[to] += amount;
        emit Transfer(address(0), to, amount);
    }

    function pause() external onlyOwner {
        paused = true;
    }

    function blockAddress(address account) external onlyOwner {
        isBlocked[account] = true;
    }

    function transfer(address to, uint256 amount) external returns (bool) {
        if (paused) revert Paused();
        if (isBlocked[msg.sender] || isBlocked[to]) revert Blocked();
        return _move(msg.sender, to, amount);
    }

    function transferFrom(address from, address to, uint256 amount) external returns (bool) {
        if (paused) revert Paused();
        if (isBlocked[from] || isBlocked[to]) revert Blocked();
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

    function _move(address from, address to, uint256 amount) internal returns (bool) {
        require(balanceOf[from] >= amount, "balance");
        balanceOf[from] -= amount;
        balanceOf[to] += amount;
        emit Transfer(from, to, amount);
        return true;
    }
}

// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

/// @notice IMPOSTOR TWIN 2 — the "looks upgradeable" variant: a minimal
/// EIP-1967 proxy that sets the IMPLEMENTATION slot (the wrong slot) and
/// leaves the beacon slot empty. `paused()` answers on its proxy just like
/// the genuine pattern, so a lazy probe pass is fooled — but the composed
/// fingerprint rejects it on F2 (impl slot SET, beacon slot EMPTY), F3 (the
/// embedded/delegated address is a token impl that does not answer
/// implementation()) and F4-blocklist (no resolved beacon to probe). The
/// twin verify prescribed: it fails ONLY because beacon-slot-set is required.
contract ImpostorSelfProxy {
    // keccak256("eip1967.proxy.implementation") - 1
    bytes32 internal constant IMPL_SLOT = 0x360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc;

    constructor(address impl) {
        assembly {
            sstore(IMPL_SLOT, impl)
        }
    }

    fallback() external payable {
        address impl;
        assembly {
            impl := sload(IMPL_SLOT)
        }
        assembly {
            calldatacopy(0, 0, calldatasize())
            let success := delegatecall(gas(), impl, 0, calldatasize(), 0, 0)
            returndatacopy(0, 0, returndatasize())
            switch success
            case 0 { revert(0, returndatasize()) }
            default { return(0, returndatasize()) }
        }
    }

    receive() external payable {}
}

/// @notice The twin's implementation — plain ERC-20 surface with name/symbol
/// in immutables (code-resident, so no init call is needed through the
/// proxy). Deliberately carries NO implementation() selector and NO
/// blocklist anywhere.
contract ImpostorSelfProxyImpl {
    address public immutable i_owner;

    string public name;
    string public symbol;
    uint8 public constant decimals = 18;
    bool public paused; // answers through the proxy (reads proxy storage) — mimicry
    uint256 public totalSupply;
    mapping(address => uint256) public balanceOf;
    mapping(address => mapping(address => uint256)) public allowance;

    event Transfer(address indexed from, address indexed to, uint256 value);
    event Approval(address indexed owner, address indexed spender, uint256 value);

    error OwnerOnly();

    constructor() {
        i_owner = msg.sender;
    }

    /// @dev storage lives in the PROXY (delegatecall), so name/symbol are set
    /// by calling init THROUGH the proxy right after deploy (deploy script).
    function init(string calldata name_, string calldata symbol_) external {
        if (msg.sender != i_owner) revert OwnerOnly();
        if (bytes(name).length != 0) revert("initialized");
        name = name_;
        symbol = symbol_;
    }

    modifier onlyOwner() {
        if (msg.sender != i_owner) revert OwnerOnly();
        _;
    }

    function mint(address to, uint256 amount) external onlyOwner {
        totalSupply += amount;
        balanceOf[to] += amount;
        emit Transfer(address(0), to, amount);
    }

    function transfer(address to, uint256 amount) external returns (bool) {
        return _move(msg.sender, to, amount);
    }

    function transferFrom(address from, address to, uint256 amount) external returns (bool) {
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

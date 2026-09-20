// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

/// Minimal ERC-20 with a global pause — the IMPL side of the genuine
/// Robinhood stock-token pattern (calibration_4663.json: pause lives with the
/// token logic; the blocklist does NOT). Deliberately NO `isBlocked`: calling
/// selector 0xfbac3951 on the impl/proxy must REVERT (calibrated behavior the
/// guard's beacon-resolution path relies on — the blocklist answers on the
/// beacon only).
///
/// Storage semantics under the genuine forwarder (delegatecall, NO proxy
/// initialization): every storage field reads zero in the proxy's context
/// until a transaction through the proxy writes it. So `paused` starts false
/// (correct default) and token state is created by `mint`ing THROUGH the
/// proxy — exactly how the genuine shared-impl layout behaves (one impl, one
/// storage context per proxy). `name`/`symbol` are per-token STORAGE set by
/// `init` through each proxy (the constructor covers the standalone "plain"
/// deployment); `admin` is an immutable baked into this code, so it is the
/// same key in every storage context. Upgrade targets (v1/v2) are the same
/// contract, so the layout is identical and balances survive an upgrade.
contract MockTokenImpl {
    string public name;
    string public symbol;
    uint8 public constant decimals = 18;

    uint256 public totalSupply;
    mapping(address account => uint256) public balanceOf;
    mapping(address owner => mapping(address spender => uint256)) public allowance;

    bool public paused;
    bool public initialized;
    address public immutable admin;

    event Transfer(address indexed from, address indexed to, uint256 value);
    event Approval(address indexed owner, address indexed spender, uint256 value);
    event PauseSet(bool paused);

    error Unauthorized(address caller);
    error Paused();
    error AlreadyInitialized();
    error InsufficientBalance(address owner, uint256 wanted);
    error AllowanceTooLow(address owner, address spender, uint256 allowed, uint256 wanted);

    modifier onlyAdmin() {
        if (msg.sender != admin) revert Unauthorized(msg.sender);
        _;
    }

    modifier whenNotPaused() {
        if (paused) revert Paused();
        _;
    }

    constructor(string memory name_, string memory symbol_) {
        admin = msg.sender;
        // standalone ("plain") deployment: its own storage context gets the
        // identity here; pattern proxies call `init` through the forwarder
        name = name_;
        symbol = symbol_;
        initialized = true;
    }

    /// Per-token identity — called THROUGH each forwarder proxy (writes the
    /// proxy's storage), once. The genuine tokens carry distinct names over
    /// one shared impl; so do these.
    function init(string calldata name_, string calldata symbol_) external onlyAdmin {
        if (initialized) revert AlreadyInitialized();
        name = name_;
        symbol = symbol_;
        initialized = true;
    }

    /// Mints token state — called THROUGH each forwarder proxy (the
    /// delegatecall writes the proxy's storage), or directly on a standalone
    /// (no-pattern) deployment.
    function mint(address to, uint256 amount) external onlyAdmin {
        unchecked {
            totalSupply += amount;
            balanceOf[to] += amount;
        }
        emit Transfer(address(0), to, amount);
    }

    /// Pause state is PER TOKEN: called through a proxy it writes that
    /// proxy's storage — the guard's GUARD_PAUSED probe (paused() on the
    /// PROXY) reads exactly this.
    function pause() external onlyAdmin {
        paused = true;
        emit PauseSet(true);
    }

    function unpause() external onlyAdmin {
        paused = false;
        emit PauseSet(false);
    }

    function transfer(address to, uint256 amount) external whenNotPaused returns (bool) {
        _move(msg.sender, to, amount);
        return true;
    }

    function approve(address spender, uint256 amount) external returns (bool) {
        allowance[msg.sender][spender] = amount;
        emit Approval(msg.sender, spender, amount);
        return true;
    }

    function transferFrom(address from, address to, uint256 amount)
        external
        whenNotPaused
        returns (bool)
    {
        uint256 allowed = allowance[from][msg.sender];
        if (allowed != type(uint256).max) {
            if (allowed < amount) revert AllowanceTooLow(from, msg.sender, allowed, amount);
            unchecked {
                allowance[from][msg.sender] = allowed - amount;
            }
        }
        _move(from, to, amount);
        return true;
    }

    function _move(address from, address to, uint256 amount) private {
        uint256 balance = balanceOf[from];
        if (balance < amount) revert InsufficientBalance(from, amount);
        unchecked {
            balanceOf[from] = balance - amount;
            balanceOf[to] += amount;
        }
        emit Transfer(from, to, amount);
    }
}

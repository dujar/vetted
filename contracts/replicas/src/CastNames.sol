// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

/// @notice Shared name/symbol for the whole demo cast — replica AND twins.
/// The twins' mimicry of these strings is the impostor trap (the real
/// impostor found in the wild copies name/ticker, knowledge
/// robinhood-stock-tokens.md:26-27); "Aurelia Industries" is a demo company,
/// honestly labelled here and in demo/assets.md, never a registry asset.
library CastNames {
    string public constant NAME = unicode"Aurelia Industries • Robinhood Token";
    string public constant SYMBOL = "AURE";
}

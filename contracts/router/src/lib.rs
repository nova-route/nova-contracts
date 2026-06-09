#![no_std]

use soroban_sdk::{contract, contractimpl, symbol_short, vec, Address, Env, Vec};

/// NovaRouter is the primary contract struct for handling multi-hop DeFi routing
/// on the Stellar Soroban platform. It orchestrates pathfinding and executes
/// token swaps across a sequence of liquidity pools.
#[contract]
pub struct NovaRouter;

/// Implementation block for NovaRouter contract functions
#[contractimpl]
impl NovaRouter {
    /// Execute a multi-hop swap route through a series of token pools.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment context for contract execution
    /// * `sender` - The Address of the account initiating the swap
    /// * `path` - A Vec<Address> representing the sequence of tokens in the swap path
    /// * `amount` - The initial i128 input amount to be swapped
    ///
    /// # Returns
    /// Returns the final i128 output amount after applying simulated pool fees
    /// and routing through all hops in the path.
    ///
    /// # Panics
    /// Will panic if the input amount is less than or equal to zero.
    pub fn execute_route(
        env: Env,
        sender: Address,
        path: Vec<Address>,
        amount: i128,
    ) -> i128 {
        // Security check: ensure amount is positive and non-zero
        if amount <= 0 {
            panic!("Route execution failed: amount must be greater than zero");
        }

        // Security check: ensure path contains at least 2 tokens (start and end)
        if path.len() < 2 {
            panic!("Route execution failed: path must contain at least 2 tokens");
        }

        // Initialize the output amount with the input amount
        let mut current_amount = amount;

        // Simulate routing through each hop in the path
        // Each hop applies a 0.3% fee to simulate real DEX pool dynamics
        let hops = path.len() - 1;
        for _hop in 0..hops {
            // Simulate pool fee deduction: 0.3% per hop
            // Fee calculation: amount * 0.003 (0.3%)
            let fee = (current_amount * 3) / 1000;
            current_amount -= fee;

            // Additional safety: prevent dust amounts from becoming zero
            if current_amount <= 0 {
                panic!("Route execution failed: insufficient amount after fees");
            }
        }

        // Emit a route execution event for on-chain tracking
        env.events().publish(
            (symbol_short!("route_ex"),),
            (sender, amount, current_amount, hops as i128),
        );

        // Return the final output amount after all routing hops and fees
        current_amount
    }

    /// Get the simulated fee for a given amount and number of hops.
    /// This is a utility function for off-chain fee estimation.
    ///
    /// # Arguments
    /// * `_env` - The Soroban environment (unused but required by contract pattern)
    /// * `amount` - The i128 amount to calculate fees for
    /// * `hops` - The number of hops in the route
    ///
    /// # Returns
    /// Returns the total i128 fee amount that would be deducted
    pub fn estimate_fee(_env: Env, amount: i128, hops: i128) -> i128 {
        if amount <= 0 || hops <= 0 {
            return 0;
        }

        // Calculate cumulative fee effect: each hop applies 0.3% fee
        // Simplified formula: total_fee ≈ amount * (0.003 ^ hops)
        // For PoC, using linear approximation: amount * 0.003 * hops
        let fee_per_hop = (amount * 3) / 1000;
        fee_per_hop * hops
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::Address as _;

    #[test]
    fn test_execute_route_single_hop() {
        // Initialize test environment
        let env = Env::default();

        // Create mock token addresses for testing
        let token_a = Address::generate(&env);
        let token_b = Address::generate(&env);
        let sender = Address::generate(&env);

        // Create path with 2 tokens (single hop)
        let path = vec![&env, token_a.clone(), token_b.clone()];

        // Execute route with 1000 units input
        let input_amount: i128 = 1000;
        let output = NovaRouter::execute_route(env, sender, path, input_amount);

        // Expected: 1000 - (1000 * 0.3%) = 1000 - 3 = 997
        let expected_output: i128 = 997;
        assert_eq!(output, expected_output);
    }

    #[test]
    fn test_execute_route_multi_hop() {
        // Initialize test environment
        let env = Env::default();

        // Create mock token addresses for multi-hop path
        let token_a = Address::generate(&env);
        let token_b = Address::generate(&env);
        let token_c = Address::generate(&env);
        let sender = Address::generate(&env);

        // Create path with 3 tokens (two hops)
        let path = vec![&env, token_a, token_b, token_c];

        // Execute route with 10000 units input
        let input_amount: i128 = 10000;
        let output = NovaRouter::execute_route(env, sender, path, input_amount);

        // Expected calculation:
        // Hop 1: 10000 - (10000 * 0.3%) = 10000 - 30 = 9970
        // Hop 2: 9970 - (9970 * 0.3%) = 9970 - 29 = 9941
        let expected_output: i128 = 9941;
        assert_eq!(output, expected_output);
    }

    #[test]
    fn test_estimate_fee() {
        let env = Env::default();

        // Test fee estimation for single hop
        let fee_single = NovaRouter::estimate_fee(env.clone(), 1000, 1);
        // Expected: (1000 * 3) / 1000 * 1 = 3
        assert_eq!(fee_single, 3);

        // Test fee estimation for two hops
        let fee_double = NovaRouter::estimate_fee(env, 1000, 2);
        // Expected: ((1000 * 3) / 1000) * 2 = 3 * 2 = 6
        assert_eq!(fee_double, 6);
    }

    #[test]
    #[should_panic(expected = "amount must be greater than zero")]
    fn test_execute_route_zero_amount() {
        let env = Env::default();
        let token_a = Address::generate(&env);
        let token_b = Address::generate(&env);
        let sender = Address::generate(&env);

        let path = vec![&env, token_a, token_b];

        // Should panic with zero amount
        NovaRouter::execute_route(env, sender, path, 0);
    }

    #[test]
    #[should_panic(expected = "amount must be greater than zero")]
    fn test_execute_route_negative_amount() {
        let env = Env::default();
        let token_a = Address::generate(&env);
        let token_b = Address::generate(&env);
        let sender = Address::generate(&env);

        let path = vec![&env, token_a, token_b];

        // Should panic with negative amount
        NovaRouter::execute_route(env, sender, path, -100);
    }

    #[test]
    #[should_panic(expected = "path must contain at least 2 tokens")]
    fn test_execute_route_insufficient_path_length() {
        let env = Env::default();
        let token_a = Address::generate(&env);
        let sender = Address::generate(&env);

        // Create path with only 1 token (invalid)
        let path = vec![&env, token_a];

        // Should panic with insufficient path length
        NovaRouter::execute_route(env, sender, path, 1000);
    }
}

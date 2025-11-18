/// A quantum channel connecting two nodes
pub struct QuantumChannel {
    /// ID of the first node
    pub node_a: usize,
    /// ID of the second node
    pub node_b: usize,
    /// Physical distance in kilometers
    pub distance_km: f64,
    /// Attenuation coefficient (dB/km) - typical: 0.2 for telecom fiber
    pub attenuation_db_per_km: f64,
}

impl QuantumChannel {
    /// Create a new quantum channel
    pub fn new(node_a: usize, node_b: usize, distance_km: f64, attenuation_db_per_km: f64) -> Self {
        QuantumChannel {
            node_a,
            node_b,
            distance_km,
            attenuation_db_per_km,
        }
    }

    /// Calculate success probability using exponential loss model
    /// p = e^(-α*L) where α is attenuation and L is distance
    pub fn success_probability(&self) -> f64 {
        // Convert dB/km to Neper/km: α = (ln(10)/10) * attenuation_dB
        let alpha = (10.0_f64.ln() / 10.0) * self.attenuation_db_per_km;
        (-alpha * self.distance_km).exp()
    }

    /// Check if this channel connects to a specific node
    pub fn connects_to(&self, node_id: usize) -> bool {
        self.node_a == node_id || self.node_b == node_id
    }

    /// Get the partner node ID (given one end of the channel)
    pub fn get_partner(&self, node_id: usize) -> Option<usize> {
        if self.node_a == node_id {
            Some(self.node_b)
        } else if self.node_b == node_id {
            Some(self.node_a)
        } else {
            None
        }
    }

    /// Attempt entanglement generation (returns true if successful based on probability)
    /// This is a simple probabilistic model - will be enhanced later
    pub fn attempt_generation(&self) -> bool {
        use rand::Rng;
        let mut rng = rand::rng();
        rng.random::<f64>() < self.success_probability()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_creation() {
        let channel = QuantumChannel::new(0, 1, 10.0, 0.2);
        assert_eq!(channel.node_a, 0);
        assert_eq!(channel.node_b, 1);
        assert_eq!(channel.distance_km, 10.0);
    }

    #[test]
    fn test_success_probability() {
        let channel = QuantumChannel::new(0, 1, 10.0, 0.2);
        let prob = channel.success_probability();

        // For 10 km with 0.2 dB/km attenuation:
        // α = ln(10)/10 * 0.2 ≈ 0.046
        // p = e^(-0.046 * 10) ≈ 0.631
        assert!(prob > 0.6 && prob < 0.65);
    }

    #[test]
    fn test_zero_distance() {
        let channel = QuantumChannel::new(0, 1, 0.0, 0.2);
        let prob = channel.success_probability();

        // Zero distance should give p ≈ 1.0
        assert!((prob - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_connects_to() {
        let channel = QuantumChannel::new(0, 1, 10.0, 0.2);
        assert!(channel.connects_to(0));
        assert!(channel.connects_to(1));
        assert!(!channel.connects_to(2));
    }

    #[test]
    fn test_get_partner() {
        let channel = QuantumChannel::new(0, 1, 10.0, 0.2);
        assert_eq!(channel.get_partner(0), Some(1));
        assert_eq!(channel.get_partner(1), Some(0));
        assert_eq!(channel.get_partner(2), None);
    }
}

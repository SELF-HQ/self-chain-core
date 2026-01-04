
use prometheus::{
    register_histogram_with_registry, register_int_counter_with_registry,
    register_int_gauge_with_registry, Histogram, IntCounter, IntGauge, Registry,
};
use std::sync::Arc;


#[derive(Debug)]
pub struct ConsensusMetrics {
    // Block metrics
    pub blocks_validated: Arc<IntCounter>,
    pub blocks_validated_total: Arc<IntCounter>,
    pub block_validation_time: Arc<Histogram>,
    pub block_validation_time_total: Arc<Histogram>,
    pub block_efficiency: Arc<IntGauge>,
    pub block_efficiency_total: Arc<IntGauge>,

    // Transaction validation metrics
    pub transactions_validated: Arc<IntCounter>,
    pub transactions_validated_total: Arc<IntCounter>,
    pub transaction_validation_time: Arc<Histogram>,
    pub transaction_validation_time_total: Arc<Histogram>,

    // Voting metrics
    pub voting_rounds_started: Arc<IntCounter>,
    pub voting_rounds_started_total: Arc<IntCounter>,
    pub votes_cast: Arc<IntCounter>,
    pub votes_cast_total: Arc<IntCounter>,
    pub voting_participation_rate: Arc<Histogram>,
    pub voting_participation_rate_total: Arc<Histogram>,
    pub voting_duration: Arc<Histogram>,
    pub voting_duration_total: Arc<Histogram>,

    // Error metrics
    pub validation_errors: Arc<IntCounter>,
    pub validation_errors_total: Arc<IntCounter>,

    // Validator metrics
    pub validator_efficiency: Arc<IntGauge>,
    pub validator_uptime: Arc<IntGauge>,
    pub validator_score: Arc<IntGauge>,

    // Network metrics
    pub peer_connections: Arc<IntGauge>,
    pub peer_connections_total: Arc<IntGauge>,
    pub message_latency: Arc<Histogram>,
    pub message_latency_total: Arc<Histogram>,
    pub message_throughput: Arc<Histogram>,
    pub message_throughput_total: Arc<Histogram>,
    pub network_errors: Arc<IntCounter>,
    pub network_errors_total: Arc<IntCounter>,
    
    // Peer metrics
    pub peer_errors: Arc<IntCounter>,
    pub peer_errors_total: Arc<IntCounter>,
    pub peer_message_count: Arc<IntCounter>,
    pub peer_message_count_total: Arc<IntCounter>,
    pub peer_latency: Arc<Histogram>,
    pub peer_latency_total: Arc<Histogram>,
    
    // System metrics
    pub memory_usage: Arc<IntGauge>,
    pub cpu_usage: Arc<Histogram>,
    pub peer_reputation_score_total: Arc<Histogram>,
    pub peer_uptime: Arc<Histogram>,
    pub peer_uptime_total: Arc<Histogram>,

    // Performance metrics
    pub ai_validation_time: Arc<Histogram>,
    pub color_transition_time: Arc<Histogram>,
}

impl ConsensusMetrics {
    pub fn new(registry: &Registry) -> Result<Self, prometheus::Error> {
        Ok(Self {
            blocks_validated: Arc::new(register_int_counter_with_registry!(
                "poai_blocks_validated",
                "Number of blocks validated",
                registry
            )?),
            blocks_validated_total: Arc::new(register_int_counter_with_registry!(
                "poai_blocks_validated_total",
                "Total number of blocks validated",
                registry
            )?),
            block_validation_time: Arc::new(register_histogram_with_registry!(
                "poai_block_validation_time_seconds",
                "Time taken to validate blocks",
                vec![0.001, 0.01, 0.1, 1.0, 10.0],
                registry
            )?),
            block_validation_time_total: Arc::new(register_histogram_with_registry!(
                "poai_block_validation_time_seconds_total",
                "Total time taken to validate blocks",
                vec![0.001, 0.01, 0.1, 1.0, 10.0],
                registry
            )?),
            block_efficiency: Arc::new(register_int_gauge_with_registry!(
                "poai_block_efficiency",
                "Block efficiency score",
                registry
            )?),
            block_efficiency_total: Arc::new(register_int_gauge_with_registry!(
                "poai_block_efficiency_total",
                "Total block efficiency score",
                registry
            )?),

            transactions_validated: Arc::new(register_int_counter_with_registry!(
                "poai_transactions_validated",
                "Number of transactions validated",
                registry
            )?),
            transactions_validated_total: Arc::new(register_int_counter_with_registry!(
                "poai_transactions_validated_total",
                "Total number of transactions validated",
                registry
            )?),
            transaction_validation_time: Arc::new(register_histogram_with_registry!(
                "poai_transaction_validation_time_seconds",
                "Time taken to validate transactions",
                vec![0.001, 0.01, 0.1, 1.0, 10.0],
                registry
            )?),
            transaction_validation_time_total: Arc::new(register_histogram_with_registry!(
                "poai_transaction_validation_time_seconds_total",
                "Total time taken to validate transactions",
                vec![0.01, 0.05, 0.1, 0.5, 1.0],
                registry
            )?),

            voting_rounds_started: Arc::new(register_int_counter_with_registry!(
                "poai_voting_rounds_started",
                "Number of voting rounds started",
                registry
            )?),
            voting_rounds_started_total: Arc::new(register_int_counter_with_registry!(
                "poai_voting_rounds_started_total",
                "Total number of voting rounds started",
                registry
            )?),
            votes_cast: Arc::new(register_int_counter_with_registry!(
                "poai_votes_cast",
                "Number of votes cast",
                registry
            )?),
            votes_cast_total: Arc::new(register_int_counter_with_registry!(
                "poai_votes_cast_total",
                "Total number of votes cast",
                registry
            )?),
            voting_participation_rate: Arc::new(register_histogram_with_registry!(
                "poai_voting_participation_rate",
                "Voting participation rate",
                vec![0.1, 0.25, 0.5, 0.75, 1.0],
                registry
            )?),
            voting_participation_rate_total: Arc::new(register_histogram_with_registry!(
                "poai_voting_participation_rate_total",
                "Total voting participation rate",
                vec![0.1, 0.25, 0.5, 0.75, 1.0],
                registry
            )?),
            voting_duration: Arc::new(register_histogram_with_registry!(
                "poai_voting_duration_seconds",
                "Voting round duration",
                vec![1.0, 5.0, 10.0, 30.0, 60.0],
                registry
            )?),
            voting_duration_total: Arc::new(register_histogram_with_registry!(
                "poai_voting_duration_seconds_total",
                "Total voting round duration",
                vec![1.0, 5.0, 10.0, 30.0, 60.0],
                registry
            )?),

            validation_errors: Arc::new(register_int_counter_with_registry!(
                "poai_validation_errors",
                "Number of validation errors",
                registry
            )?),
            validation_errors_total: Arc::new(register_int_counter_with_registry!(
                "poai_validation_errors_total",
                "Total number of validation errors",
                registry
            )?),

            validator_efficiency: Arc::new(register_int_gauge_with_registry!(
                "poai_validator_efficiency",
                "Validator efficiency score",
                registry
            )?),
            validator_uptime: Arc::new(register_int_gauge_with_registry!(
                "poai_validator_uptime",
                "Validator uptime percentage",
                registry
            )?),
            validator_score: Arc::new(register_int_gauge_with_registry!(
                "poai_validator_score",
                "Validator reputation score",
                registry
            )?),

            peer_connections: Arc::new(register_int_gauge_with_registry!(
                "poai_peer_connections",
                "Number of active peer connections",
                registry
            )?),
            peer_connections_total: Arc::new(register_int_gauge_with_registry!(
                "poai_peer_connections_total",
                "Total number of peer connections",
                registry
            )?),
            peer_message_count: Arc::new(register_int_counter_with_registry!(
                "poai_peer_messages",
                "Number of peer messages",
                registry
            )?),
            peer_message_count_total: Arc::new(register_int_counter_with_registry!(
                "poai_peer_messages_total",
                "Total number of peer messages",
                registry
            )?),
            peer_latency: Arc::new(register_histogram_with_registry!(
                "poai_peer_latency_seconds",
                "Peer message latency",
                vec![0.001, 0.01, 0.1, 1.0, 10.0],
                registry
            )?),
            peer_latency_total: Arc::new(register_histogram_with_registry!(
                "poai_peer_latency_seconds_total",
                "Total peer message latency",
                vec![0.001, 0.01, 0.1, 1.0, 10.0],
                registry
            )?),
            peer_errors: Arc::new(register_int_counter_with_registry!(
                "poai_peer_errors",
                "Number of peer errors",
                registry
            )?),
            peer_errors_total: Arc::new(register_int_counter_with_registry!(
                "poai_peer_errors_total",
                "Total number of peer errors",
                registry
            )?),

            peer_reputation_score_total: Arc::new(register_histogram_with_registry!(
                "poai_peer_reputation_score_total",
                "Total peer reputation score",
                vec![0.0, 0.25, 0.5, 0.75, 1.0],
                registry
            )?),
            peer_uptime: Arc::new(register_histogram_with_registry!(
                "poai_peer_uptime_seconds",
                "Peer uptime",
                vec![3600.0, 86400.0, 604800.0, 2678400.0], // 1h, 24h, 7d, 31d
                registry
            )?),
            peer_uptime_total: Arc::new(register_histogram_with_registry!(
                "poai_peer_uptime_seconds_total",
                "Total peer uptime",
                vec![3600.0, 86400.0, 604800.0, 2678400.0], // 1h, 24h, 7d, 31d
                registry
            )?),

            message_latency: Arc::new(register_histogram_with_registry!(
                "poai_message_latency_seconds",
                "Message processing latency",
                vec![0.001, 0.01, 0.1, 1.0, 10.0],
                registry
            )?),
            message_latency_total: Arc::new(register_histogram_with_registry!(
                "poai_message_latency_seconds_total",
                "Total message processing latency",
                vec![0.001, 0.01, 0.1, 1.0, 10.0],
                registry
            )?),
            message_throughput: Arc::new(register_histogram_with_registry!(
                "poai_message_throughput",
                "Message processing throughput",
                vec![1.0, 10.0, 100.0, 1000.0, 10000.0],
                registry
            )?),
            message_throughput_total: Arc::new(register_histogram_with_registry!(
                "poai_message_throughput_total",
                "Total message processing throughput",
                vec![1.0, 10.0, 100.0, 1000.0, 10000.0],
                registry
            )?),

            ai_validation_time: Arc::new(register_histogram_with_registry!(
                "poai_ai_validation_time_seconds",
                "Time taken for AI validation",
                vec![0.1, 0.5, 1.0, 5.0, 10.0],
                registry
            )?),
            color_transition_time: Arc::new(register_histogram_with_registry!(
                "poai_color_transition_time_seconds",
                "Time taken for color transitions",
                vec![0.001, 0.01, 0.1, 1.0, 10.0],
                registry
            )?),
            network_errors: Arc::new(register_int_counter_with_registry!(
                "poai_network_errors",
                "Number of network errors",
                registry
            )?),
            network_errors_total: Arc::new(register_int_counter_with_registry!(
                "poai_network_errors_total",
                "Total number of network errors",
                registry
            )?),
            memory_usage: Arc::new(register_int_gauge_with_registry!(
                "poai_memory_usage_bytes",
                "Memory usage in bytes",
                registry
            )?),
            cpu_usage: Arc::new(register_histogram_with_registry!(
                "poai_cpu_usage_percent",
                "CPU usage percentage",
                vec![10.0, 25.0, 50.0, 75.0, 90.0],
                registry
            )?),
        })
    }

    // Voting metrics methods
    pub fn increment_voting_rounds_started(&self) {
        self.voting_rounds_started.inc();
        self.voting_rounds_started_total.inc();
    }

    pub fn increment_votes_cast(&self) {
        self.votes_cast.inc();
        self.votes_cast_total.inc();
    }

    pub fn observe_voting_participation_rate(&self, rate: f64) {
        self.voting_participation_rate.observe(rate);
        self.voting_participation_rate_total.observe(rate);
    }

    pub fn observe_voting_round_duration(&self, duration: f64) {
        self.voting_duration.observe(duration);
        self.voting_duration_total.observe(duration);
    }

    pub fn set_active_validators(&self, count: u64) {
        // Use peer_connections as a proxy for active validators
        self.peer_connections.set(count as i64);
    }

    pub fn increment_voting_error(&self) {
        self.validation_errors.inc();
        self.validation_errors_total.inc();
    }

    pub fn increment_network_error(&self) {
        self.peer_errors.inc();
        self.peer_errors_total.inc();
    }

    // Block validation methods
    pub fn increment_blocks_validated(&self) {
        self.blocks_validated.inc();
        self.blocks_validated_total.inc();
    }

    pub fn observe_block_validation_time(&self, duration: f64) {
        self.block_validation_time.observe(duration);
        self.block_validation_time_total.observe(duration);
    }

    pub fn set_block_efficiency(&self, efficiency: f64) {
        self.block_efficiency.set(efficiency as i64);
        self.block_efficiency_total.set(efficiency as i64);
    }

    // Transaction validation methods
    pub fn increment_transactions_validated(&self) {
        self.transactions_validated.inc();
        self.transactions_validated_total.inc();
    }

    pub fn observe_transaction_validation_time(&self, duration: f64) {
        self.transaction_validation_time.observe(duration);
        self.transaction_validation_time_total.observe(duration);
    }

    // Validator metrics methods
    pub fn set_validator_efficiency(&self, efficiency: i64) {
        self.validator_efficiency.set(efficiency);
    }

    pub fn set_validator_uptime(&self, uptime: i64) {
        self.validator_uptime.set(uptime);
    }

    pub fn set_validator_score(&self, score: i64) {
        self.validator_score.set(score);
    }

    // Peer metrics methods
    pub fn increment_peer_messages(&self) {
        self.peer_message_count.inc();
        self.peer_message_count_total.inc();
    }

    pub fn observe_peer_latency(&self, latency: f64) {
        self.peer_latency.observe(latency);
        self.peer_latency_total.observe(latency);
    }

    pub fn increment_peer_errors(&self) {
        self.peer_errors.inc();
        self.peer_errors_total.inc();
    }

    pub fn observe_peer_reputation_score(&self, score: f64) {
        // Using only total since peer_reputation_score field was removed
        self.peer_reputation_score_total.observe(score);
    }

    pub fn observe_peer_uptime(&self, uptime: f64) {
        self.peer_uptime.observe(uptime);
        self.peer_uptime_total.observe(uptime);
    }

    // Performance metrics methods
    pub fn observe_ai_validation_time(&self, duration: f64) {
        self.ai_validation_time.observe(duration);
    }

    pub fn observe_color_transition_time(&self, duration: f64) {
        self.color_transition_time.observe(duration);
    }

    pub fn set_memory_usage(&self, usage: i64) {
        self.memory_usage.set(usage);
    }

    pub fn observe_cpu_usage(&self, usage: f64) {
        self.cpu_usage.observe(usage);
    }

    pub fn observe_block_validation(&self, duration: f64) {
        self.block_validation_time.observe(duration);
        self.block_validation_time_total.observe(duration);
    }

    pub fn observe_transaction_validation(&self, duration: f64) {
        self.transaction_validation_time.observe(duration);
        self.transaction_validation_time_total.observe(duration);
    }


    pub fn observe_voting_duration(&self, duration: f64) {
        self.voting_duration.observe(duration);
        self.voting_duration_total.observe(duration);
    }

    pub fn observe_message_latency(&self, latency: f64) {
        self.message_latency.observe(latency);
        self.message_latency_total.observe(latency);
    }

    pub fn observe_message_throughput(&self, throughput: f64) {
        self.message_throughput.observe(throughput);
        self.message_throughput_total.observe(throughput);
    }

    pub fn increment_validation_error(&self) {
        self.validation_errors.inc();
        self.validation_errors_total.inc();
    }

    pub fn increment_validation_failures(&self, _reason: &str) {
        self.validation_errors.inc();
        self.validation_errors_total.inc();
    }

    pub fn increment_valid_transactions(&self) {
        self.transactions_validated.inc();
        self.transactions_validated_total.inc();
    }

    pub fn increment_wallet_updates(&self) {
        // Using network errors as placeholder for wallet updates
        self.network_errors.inc();
        self.network_errors_total.inc();
    }

}

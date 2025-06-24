/**
 * API Client for FastSSSP Rust Backend
 */
class FastSSSPAPI {
    constructor(baseUrl = 'http://localhost:3005') {
        this.baseUrl = baseUrl;
        this.currentSessionId = null;
    }

    /**
     * Generate a new graph
     */
    async generateGraph(graphType, nodeCount, options = {}) {
        const request = {
            graph_type: graphType,
            node_count: nodeCount,
            edges_per_node: options.edgesPerNode || 3,
            radius: options.radius || 0.2,
            grid_dimensions: options.gridDimensions
        };

        try {
            const response = await fetch(`${this.baseUrl}/api/graphs/generate`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(request)
            });

            if (!response.ok) {
                const error = await response.json();
                throw new Error(error.message || 'Failed to generate graph');
            }

            const session = await response.json();
            this.currentSessionId = session.id;
            return session;
        } catch (error) {
            console.error('Error generating graph:', error);
            throw error;
        }
    }

    /**
     * Get graph data for current session
     */
    async getGraph(sessionId = null) {
        const id = sessionId || this.currentSessionId;
        if (!id) {
            throw new Error('No active session');
        }

        try {
            const response = await fetch(`${this.baseUrl}/api/graphs/${id}`);
            
            if (!response.ok) {
                const error = await response.json();
                throw new Error(error.message || 'Failed to get graph');
            }

            return await response.json();
        } catch (error) {
            console.error('Error getting graph:', error);
            throw error;
        }
    }

    /**
     * Run an algorithm on the current graph
     */
    async runAlgorithm(algorithm, source, options = {}) {
        if (!this.currentSessionId) {
            throw new Error('No active session');
        }

        const request = {
            algorithm,
            source,
            k: options.k,
            t: options.t,
            auto_tune: options.autoTune || false,
            skip_sweep: options.skipSweep || false,
            parallel: options.parallel || false
        };

        try {
            const response = await fetch(`${this.baseUrl}/api/algorithms/run/${this.currentSessionId}`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(request)
            });

            if (!response.ok) {
                const error = await response.json();
                throw new Error(error.message || 'Algorithm execution failed');
            }

            return await response.json();
        } catch (error) {
            console.error('Error running algorithm:', error);
            throw error;
        }
    }

    /**
     * Compare multiple algorithms
     */
    async compareAlgorithms(algorithms) {
        if (!this.currentSessionId) {
            throw new Error('No active session');
        }

        try {
            const response = await fetch(`${this.baseUrl}/api/algorithms/compare/${this.currentSessionId}`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(algorithms)
            });

            if (!response.ok) {
                const error = await response.json();
                throw new Error(error.message || 'Algorithm comparison failed');
            }

            return await response.json();
        } catch (error) {
            console.error('Error comparing algorithms:', error);
            throw error;
        }
    }

    /**
     * Run comprehensive benchmark
     */
    async runBenchmark(config) {
        try {
            const response = await fetch(`${this.baseUrl}/api/benchmark`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(config)
            });

            if (!response.ok) {
                const error = await response.json();
                throw new Error(error.message || 'Benchmark failed');
            }

            return await response.json();
        } catch (error) {
            console.error('Error running benchmark:', error);
            throw error;
        }
    }

    /**
     * Get health status
     */
    async getHealth() {
        try {
            const response = await fetch(`${this.baseUrl}/api/health`);
            return await response.json();
        } catch (error) {
            console.error('Error checking health:', error);
            throw error;
        }
    }

    /**
     * List all sessions
     */
    async listSessions() {
        try {
            const response = await fetch(`${this.baseUrl}/api/sessions`);
            return await response.json();
        } catch (error) {
            console.error('Error listing sessions:', error);
            throw error;
        }
    }

    /**
     * Get session information
     */
    async getSession(sessionId = null) {
        const id = sessionId || this.currentSessionId;
        if (!id) {
            throw new Error('No session ID provided');
        }

        try {
            const response = await fetch(`${this.baseUrl}/api/sessions/${id}`);
            
            if (!response.ok) {
                const error = await response.json();
                throw new Error(error.message || 'Failed to get session');
            }

            return await response.json();
        } catch (error) {
            console.error('Error getting session:', error);
            throw error;
        }
    }
}

// Export for use in other modules
window.FastSSSPAPI = FastSSSPAPI;

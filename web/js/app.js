/**
 * Main Application Logic for FastSSSP Web Interface
 */
class FastSSSPApp {
    constructor() {
        this.api = new FastSSSPAPI();
        this.visualizer = null;
        this.currentGraph = null;
        this.isRunning = false;
        this.charts = {};
        
        this.initializeApp();
    }

    async initializeApp() {
        try {
            // Initialize visualizer first (doesn't depend on backend)
            this.visualizer = new GraphVisualizer('graph-container');
            
            // Initialize event listeners
            this.setupEventListeners();
            
            // Initialize charts
            this.initializeCharts();
            
            // Check if backend is available
            try {
                await this.api.getHealth();
                this.showStatus('✅ Connected to FastSSSP backend', 'success');
            } catch (apiError) {
                this.showStatus('❌ Failed to connect to backend. Make sure the Rust server is running.', 'error');
                console.error('Failed to connect to backend:', apiError);
                
                // Set a retry button
                const statusElement = document.getElementById('status-message');
                if (statusElement) {
                    const retryButton = document.createElement('button');
                    retryButton.textContent = 'Retry Connection';
                    retryButton.className = 'btn btn-sm btn-warning ms-2';
                    retryButton.addEventListener('click', async () => {
                        try {
                            await this.api.getHealth();
                            this.showStatus('✅ Connected to FastSSSP backend', 'success');
                            statusElement.removeChild(retryButton);
                        } catch (retryError) {
                            this.showStatus('❌ Still unable to connect. Check server status.', 'error');
                        }
                    });
                    statusElement.appendChild(retryButton);
                }
            }
        } catch (error) {
            this.showStatus('❌ Failed to initialize application', 'error');
            console.error('Failed to initialize app:', error);
        }
    }

    setupEventListeners() {
        // Graph generation
        document.getElementById('generate-btn').addEventListener('click', () => this.generateGraph());
        
        // Algorithm execution
        document.getElementById('run-btn').addEventListener('click', () => this.runAlgorithm());
        
        // Algorithm comparison
        document.getElementById('compare-btn').addEventListener('click', () => this.compareAlgorithms());
        
        // Benchmark
        document.getElementById('benchmark-btn').addEventListener('click', () => this.runBenchmark());
        
        // Parameter changes
        document.getElementById('algorithm-select').addEventListener('change', () => this.updateParameterVisibility());
        
        // Visualization theme changes
        const themeSelector = document.getElementById('visualization-theme');
        if (themeSelector) {
            themeSelector.addEventListener('change', () => {
                const selectedTheme = themeSelector.value;
                if (this.visualizer) {
                    this.visualizer.setVisualizationTheme(selectedTheme);
                }
            });
        }
        
        // Auto-tune toggle
        document.getElementById('auto-tune').addEventListener('change', (e) => {
            const parameterInputs = document.querySelectorAll('#k-param, #t-param');
            parameterInputs.forEach(input => {
                input.disabled = e.target.checked;
            });
        });
    }

    async generateGraph() {
        if (this.isRunning) return;
        
        try {
            this.setRunning(true);
            this.showStatus('🔄 Generating graph...', 'info');
            
            const graphType = document.getElementById('graph-type').value;
            const nodeCount = parseInt(document.getElementById('node-count').value);
            const edgesPerNode = parseInt(document.getElementById('edges-per-node').value);
            
            const options = {
                edgesPerNode,
                radius: 0.2
            };
            
            // Add grid dimensions for 3D grid
            if (graphType === 'grid-3d') {
                const size = Math.ceil(Math.cbrt(nodeCount));
                options.gridDimensions = [size, size, size];
            }
            
            const session = await this.api.generateGraph(graphType, nodeCount, options);
            this.currentGraph = session.graph;
            
            // Visualize the graph
            this.visualizer.setGraph(this.currentGraph);
            
            this.showStatus(`✅ Generated ${graphType} graph with ${nodeCount} nodes`, 'success');
            
            // Update graph info
            this.updateGraphInfo();
            
        } catch (error) {
            this.showStatus(`❌ Error generating graph: ${error.message}`, 'error');
        } finally {
            this.setRunning(false);
        }
    }

    async runAlgorithm() {
        if (this.isRunning || !this.currentGraph) return;
        
        try {
            this.setRunning(true);
            const algorithm = document.getElementById('algorithm-select').value;
            const source = parseInt(document.getElementById('source-node').value) || 0;
            
            this.showStatus(`🔄 Running ${algorithm} from node ${source}...`, 'info');
            
            const options = {
                autoTune: document.getElementById('auto-tune').checked,
                skipSweep: document.getElementById('skip-sweep').checked,
                parallel: document.getElementById('parallel').checked
            };
            
            // Add manual parameters if not auto-tuning
            if (!options.autoTune) {
                const k = parseInt(document.getElementById('k-param').value);
                const t = parseInt(document.getElementById('t-param').value);
                if (k) options.k = k;
                if (t) options.t = t;
            }
            
            const result = await this.api.runAlgorithm(algorithm, source, options);
            
            // Visualize results
            await this.visualizeAlgorithmResult(result);
            
            // Update metrics
            this.updateMetrics(result);
            
            this.showStatus(`✅ ${algorithm} completed in ${result.execution_time_ms.toFixed(2)}ms`, 'success');
            
        } catch (error) {
            this.showStatus(`❌ Error running algorithm: ${error.message}`, 'error');
        } finally {
            this.setRunning(false);
        }
    }

    async compareAlgorithms() {
        if (this.isRunning || !this.currentGraph) return;
        
        try {
            this.setRunning(true);
            this.showStatus('🔄 Comparing algorithms...', 'info');
            
            const source = parseInt(document.getElementById('source-node').value) || 0;
            const algorithms = ['dijkstra', 'fast-sssp', 'mini-bmssp', 'smart-sssp'];
            
            const requests = algorithms.map(algorithm => ({
                algorithm,
                source,
                auto_tune: true
            }));
            
            const results = await this.api.compareAlgorithms(requests);
            
            // Update comparison chart
            this.updateComparisonChart(results);
            
            this.showStatus('✅ Algorithm comparison completed', 'success');
            
        } catch (error) {
            this.showStatus(`❌ Error comparing algorithms: ${error.message}`, 'error');
        } finally {
            this.setRunning(false);
        }
    }

    async runBenchmark() {
        if (this.isRunning) return;
        
        try {
            this.setRunning(true);
            this.showStatus('🔄 Running comprehensive benchmark...', 'info');
            
            const config = {
                algorithms: ['dijkstra', 'fast-sssp', 'mini-bmssp', 'smart-sssp'],
                graph_types: ['scale-free', 'grid-3d', 'geometric-3d'],
                node_counts: [100, 500, 1000, 2000],
                iterations: 3,
                timeout_seconds: 60
            };
            
            const result = await this.api.runBenchmark(config);
            
            // Update benchmark charts
            this.updateBenchmarkCharts(result);
            
            this.showStatus('✅ Benchmark completed', 'success');
            
        } catch (error) {
            this.showStatus(`❌ Error running benchmark: ${error.message}`, 'error');
        } finally {
            this.setRunning(false);
        }
    }

    async visualizeAlgorithmResult(result) {
        // Reset visualization
        this.visualizer.reset();
        
        // Update node distances
        for (const [nodeId, distance] of Object.entries(result.distances)) {
            this.visualizer.updateNodeDistance(parseInt(nodeId), distance);
        }
        
        // Animate algorithm steps
        if (result.animation_steps && result.animation_steps.length > 0) {
            await this.visualizer.animateSteps(result.animation_steps);
        }
        
        // Highlight shortest path tree
        this.visualizer.highlightShortestPaths(result.predecessors);
    }

    updateMetrics(result) {
        document.getElementById('execution-time').textContent = `${result.execution_time_ms.toFixed(2)} ms`;
        document.getElementById('nodes-processed').textContent = result.nodes_processed;
        document.getElementById('edges-relaxed').textContent = result.edges_relaxed;
        document.getElementById('memory-usage').textContent = `${result.metrics.memory_usage_mb.toFixed(2)} MB`;
        
        // Update performance chart
        this.updatePerformanceChart(result);
    }

    updateGraphInfo() {
        if (!this.currentGraph) return;
        
        document.getElementById('graph-nodes').textContent = this.currentGraph.nodes.length;
        document.getElementById('graph-edges').textContent = this.currentGraph.links.length;
        
        const avgDegree = (this.currentGraph.links.length * 2) / this.currentGraph.nodes.length;
        document.getElementById('graph-avg-degree').textContent = avgDegree.toFixed(2);
    }

    updateParameterVisibility() {
        const algorithm = document.getElementById('algorithm-select').value;
        const parameterSection = document.getElementById('parameter-section');
        
        if (algorithm === 'mini-bmssp' || algorithm === 'fast-sssp') {
            parameterSection.style.display = 'block';
        } else {
            parameterSection.style.display = 'none';
        }
    }

    initializeCharts() {
        // Performance chart
        const performanceChart = document.getElementById('performance-chart');
        if (performanceChart) {
            const performanceCtx = performanceChart.getContext('2d');
            this.charts.performance = new Chart(performanceCtx, {
            type: 'bar',
            data: {
                labels: ['Execution Time', 'Nodes Processed', 'Edges Relaxed'],
                datasets: [{
                    label: 'Current Run',
                    data: [0, 0, 0],
                    backgroundColor: ['#3498db', '#2ecc71', '#e74c3c']
                }]
            },
            options: {
                responsive: true,
                scales: {
                    y: {
                        beginAtZero: true
                    }
                }
            }
        });
        
        // Comparison chart
        const comparisonCtx = document.getElementById('comparison-chart').getContext('2d');
        this.charts.comparison = new Chart(comparisonCtx, {
            type: 'bar',
            data: {
                labels: [],
                datasets: [{
                    label: 'Execution Time (ms)',
                    data: [],
                    backgroundColor: '#3498db'
                }]
            },
            options: {
                responsive: true,
                scales: {
                    y: {
                        beginAtZero: true
                    }
                }
            }
        });
    }

    updatePerformanceChart(result) {
        this.charts.performance.data.datasets[0].data = [
            result.execution_time_ms,
            result.nodes_processed,
            result.edges_relaxed
        ];
        this.charts.performance.update();
    }

    updateComparisonChart(results) {
        const labels = results.map(r => r.algorithm);
        const data = results.map(r => r.execution_time_ms);
        
        this.charts.comparison.data.labels = labels;
        this.charts.comparison.data.datasets[0].data = data;
        this.charts.comparison.update();
    }

    updateBenchmarkCharts(benchmark) {
        // Group results by algorithm
        const algorithmData = {};
        benchmark.results.forEach(result => {
            if (!algorithmData[result.algorithm]) {
                algorithmData[result.algorithm] = [];
            }
            algorithmData[result.algorithm].push(result.execution_time_ms);
        });
        
        // Update comparison chart with benchmark data
        const labels = Object.keys(algorithmData);
        const data = labels.map(alg => {
            const times = algorithmData[alg];
            return times.reduce((sum, time) => sum + time, 0) / times.length;
        });
        
        this.charts.comparison.data.labels = labels;
        this.charts.comparison.data.datasets[0].data = data;
        this.charts.comparison.update();
    }

    setRunning(running) {
        this.isRunning = running;
        const buttons = document.querySelectorAll('button');
        buttons.forEach(btn => {
            btn.disabled = running;
        });
        
        if (running) {
            document.body.style.cursor = 'wait';
        } else {
            document.body.style.cursor = 'default';
        }
    }

    showStatus(message, type = 'info') {
        const statusElement = document.getElementById('status-message');
        statusElement.textContent = message;
        statusElement.className = `status-message ${type}`;
        
        // Auto-hide after 5 seconds for success/info messages
        if (type === 'success' || type === 'info') {
            setTimeout(() => {
                statusElement.textContent = '';
                statusElement.className = 'status-message';
            }, 5000);
        }
    }
}

// Initialize the application when the page loads
document.addEventListener('DOMContentLoaded', () => {
    window.app = new FastSSSPApp();
});

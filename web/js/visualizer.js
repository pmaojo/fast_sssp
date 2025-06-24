/**
 * Graph Visualizer Module
 * Handles visualization of graphs and algorithm execution using WebGPU
 */
class GraphVisualizer {
    constructor(containerId) {
        this.containerId = containerId;
        this.graph = null;
        this.forceGraph = null;
        this.currentAlgorithm = null;
        this.animationSpeed = 1;
        this.isAnimating = false;
        this.animationStep = 0;
        this.animationSteps = [];
        this.sourceNode = 0;
        this.visualizationTheme = 'default';
        
        this.nodeColors = {
            default: '#3498db',
            source: '#e74c3c',
            visited: '#2ecc71',
            frontier: '#f39c12',
            pivot: '#9b59b6',
            current: '#e67e22'
        };
        
        // Theme configurations
        this.themes = {
            default: {
                groundColor: '#f0f0f0',
                nodeColor: '#aaaaaa',
                linkColor: '#888888',
                pathColor: '#ffaa00',
                particleColor: '#ffff00'
            },
            maze: {
                groundColor: '#e0e0e0',
                nodeColor: '#cccccc',
                linkColor: '#666666',
                pathColor: '#ff5500',
                particleColor: '#ffff00',
                buildingRatio: 0.4,
                buildingColor: '#333333'
            },
            city: {
                groundColor: '#d0d0d0',
                nodeColor: '#bbbbbb',
                linkColor: '#777777',
                pathColor: '#ff3300',
                particleColor: '#ffcc00',
                buildingRatio: 0.6,
                buildingColor: '#555555'
            },
            farm: {
                groundColor: '#c8e6c9',
                nodeColor: '#81c784',
                linkColor: '#a5d6a7',
                pathColor: '#4caf50',
                particleColor: '#8bc34a',
                buildingRatio: 0.2,
                buildingColor: '#795548'
            },
            forest: {
                groundColor: '#004d40',
                nodeColor: '#00796b',
                linkColor: '#00695c',
                pathColor: '#4db6ac',
                particleColor: '#b2dfdb',
                buildingRatio: 0.7,
                buildingColor: '#33691e'
            }
        };
        
        // Check if WebGPU is supported
        this.supportsWebGPU = this.checkWebGPUSupport();
        console.log(`WebGPU support: ${this.supportsWebGPU ? 'Yes' : 'No'}`);
        
        // Initialize empty graph container
        this.initializeGraph();
    }
    
    /**
     * Check if WebGPU is supported in the current browser
     */
    checkWebGPUSupport() {
        return navigator.gpu !== undefined;
    }
    
    /**
     * Setup zoom controls for the graph
     */
    setupZoomControls() {
        const zoomIn = document.getElementById('zoomIn');
        const zoomOut = document.getElementById('zoomOut');
        const resetView = document.getElementById('resetView');
        
        if (zoomIn) {
            zoomIn.addEventListener('click', () => {
                if (this.forceGraph) {
                    this.forceGraph.cameraPosition({ z: this.forceGraph.cameraPosition().z * 0.8 });
                }
            });
        }
        
        if (zoomOut) {
            zoomOut.addEventListener('click', () => {
                if (this.forceGraph) {
                    this.forceGraph.cameraPosition({ z: this.forceGraph.cameraPosition().z * 1.2 });
                }
            });
        }
        
        if (resetView) {
            resetView.addEventListener('click', () => {
                if (this.forceGraph) {
                    if (this.supportsWebGPU) {
                        this.forceGraph.cameraPosition({ x: 0, y: 150, z: 10 }, { x: 0, y: 0, z: 0 }, 1000);
                    } else {
                        this.forceGraph.cameraPosition({ x: 0, y: 0, z: 150 }, { x: 0, y: 0, z: 0 }, 1000);
                    }
                }
            });
        }
    }
    
    /**
     * Initialize the force-directed graph visualization
     */
    initializeGraph() {
        const container = document.getElementById(this.containerId);
        if (!container) {
            console.error('Container not found:', this.containerId);
            return;
        }
        
        // Limpiar el contenedor
        container.innerHTML = '';
        
        console.log('Initializing graph visualization');
        
        try {
            // Crear el visualizador de grafo 3D
            this.forceGraph = ForceGraph3D()(container);
            
            // Inicializar con grafo vacío si no hay uno
            const emptyGraph = {nodes: [], links: []};
            this.forceGraph.graphData(this.graph || emptyGraph);
            
            // Configurar la apariencia del grafo
            this.forceGraph
                .nodeVal(node => node.val || 8)
                .nodeColor(node => node.color || (node.isBuilding ? '#555555' : '#aaaaaa'))
                .linkWidth(link => link.width || (link.isPath ? 4 : 1))
                .linkColor(link => link.color || (link.isPath ? '#ffaa00' : '#888888'))
                .linkDirectionalParticles(link => link.isPath ? 8 : 0)
                .linkDirectionalParticleWidth(link => link.particleWidth || 3)
                .linkDirectionalParticleSpeed(link => link.particleSpeed || 0.01)
                .linkDirectionalParticleColor(link => link.particleColor || '#ffff00')
                .onNodeClick(this.handleNodeClick.bind(this));
            
            // Añadir objetos 3D para edificios
            this.forceGraph.nodeThreeObject(node => {
                if (node.isBuilding) {
                    const height = 5 + Math.random() * 10;
                    const geometry = new THREE.BoxGeometry(3, 3, height);
                    const material = new THREE.MeshLambertMaterial({ 
                        color: node.color || '#555555',
                        transparent: true,
                        opacity: 0.8
                    });
                    const mesh = new THREE.Mesh(geometry, material);
                    mesh.position.y = height/2; // Posicionar edificio en el suelo
                    return mesh;
                }
                return null; // Usar esferas por defecto para nodos que no son edificios
            });
            
            // Establecer vista desde arriba para visualización de ciudad/laberinto
            this.forceGraph.cameraPosition({ x: 0, y: 150, z: 10 });
            
            // Añadir iluminación
            const ambientLight = new THREE.AmbientLight(0xffffff, 0.6);
            this.forceGraph.scene().add(ambientLight);
            
            const directionalLight = new THREE.DirectionalLight(0xffffff, 0.8);
            directionalLight.position.set(0, 1, 1);
            this.forceGraph.scene().add(directionalLight);
            
            // Añadir plano de suelo
            const theme = this.themes[this.visualizationTheme] || this.themes.default;
            const groundGeometry = new THREE.PlaneGeometry(500, 500);
            const groundMaterial = new THREE.MeshLambertMaterial({ 
                color: theme.groundColor,
                side: THREE.DoubleSide
            });
            const ground = new THREE.Mesh(groundGeometry, groundMaterial);
            ground.rotation.x = Math.PI / 2;
            ground.position.y = -0.5;
            this.forceGraph.scene().add(ground);
            
            // Disable physics simulation after initial layout
            setTimeout(() => {
                this.forceGraph.d3Force('charge').strength(-120); // Reduce repulsion
                this.forceGraph.d3Force('link').distance(15); // Set link distance for grid-like layout
                
                // Freeze positions after brief layout adjustment
                setTimeout(() => {
                    this.forceGraph.d3Force('center', null);
                    this.forceGraph.d3Force('charge', null);
                }, 2000);
            }, 100);
            
            // Add zoom controls
            this.setupZoomControls();
            
            console.log('Graph visualization initialized successfully');
        } catch (error) {
            console.error('Error initializing graph visualization:', error);
        }
    }
    
    /**
     * Handle node click event
     * @param {Object} node - The clicked node
     */
    handleNodeClick(node) {
        this.sourceNode = node.id;
        document.getElementById('sourceNode').value = node.id;
        this.updateNodeColors();
    }
    
    /**
     * Set the graph data for visualization
     * @param {Object} graph - Graph data with nodes and links
     */
    setGraph(graph) {
        this.graph = graph;
        
        // Get current theme settings
        const theme = this.themes[this.visualizationTheme] || this.themes.default;
        
        // Generate coordinates for themed layout
        // For a more structured layout, we'll use a grid-like arrangement
        const nodeCount = graph.nodes.length;
        const gridSize = Math.ceil(Math.sqrt(nodeCount));
        
        // Create a themed layout
        this.graph.nodes.forEach((node, index) => {
            // Calculate grid position
            const row = Math.floor(index / gridSize);
            const col = index % gridSize;
            
            // Assign coordinates with some jitter for natural appearance
            const jitterX = (Math.random() - 0.5) * 0.5;
            const jitterY = (Math.random() - 0.5) * 0.5;
            
            // Set node coordinates in a grid pattern
            node.x = col * 5 + jitterX;
            node.y = row * 5 + jitterY;
            
            // Add z-coordinate based on theme
            if (this.visualizationTheme === 'forest') {
                // For forest, add varying heights
                node.z = Math.random() * 2;
            } else if (this.visualizationTheme === 'farm') {
                // For farm, add slight terrain variations
                node.z = Math.random() * 0.5;
            } else {
                // For other themes, keep mostly flat
                node.z = 0;
            }
            
            // Set node appearance
            node.val = 8; // Node size
            node.color = theme.nodeColor; // Theme-based color
            
            // Add themed properties
            const buildingRatio = theme.buildingRatio || 0.3;
            node.isBuilding = Math.random() > (1 - buildingRatio); // Some nodes are buildings/obstacles
            if (node.isBuilding) {
                node.color = theme.buildingColor || '#555555';
                node.val = 12;
            }
        });
        
        // Make links look like paths/streets based on theme
        this.graph.links.forEach(link => {
            link.width = 2;
            link.color = theme.linkColor;
        });
        
        // Initialize the force graph
        this.initializeGraph();
        
        // Reset source node
        this.sourceNode = null;
    }
    
    /**
     * Update node colors based on their states
     */
    updateNodeColors() {
        // For 3D-Force-Graph, we need to refresh the entire scene
        if (this.graph) {
            this.forceGraph.refresh();
        }
    }
    
    /**
     * Reset all nodes and links to their default state
     */
    resetGraphState() {
        if (!this.graph) return;
        
        this.graph.nodes.forEach(node => {
            node.isVisited = false;
            node.isFrontier = false;
            node.isPivot = false;
            node.isCurrent = false;
            node.distance = Infinity;
        });
        
        this.graph.links.forEach(link => {
            link.isPath = false;
        });
        
        this.updateNodeColors();
    }
    
    /**
     * Set the animation steps for algorithm visualization
     * @param {Array} steps - Array of animation steps
     */
    setAnimationSteps(steps) {
        this.animationSteps = steps;
        this.animationStep = 0;
        this.updateProgressChart();
        
        // For 3D visualization, we might want to enhance the animation
        if (this.supportsWebGPU) {
            // Add any WebGPU-specific animation enhancements here
            console.log(`Prepared ${steps.length} animation steps for WebGPU rendering`);
        }
    }
    
    /**
     * Play the algorithm animation
     */
    playAnimation() {
        if (!this.animationSteps.length || this.isAnimating) return;
        
        this.isAnimating = true;
        
        // For WebGPU, we might want to optimize the animation
        if (this.supportsWebGPU) {
            // WebGPU can handle more complex animations
            this.animationSpeed = Math.min(this.animationSpeed * 1.5, 5);
            console.log(`Using enhanced animation speed: ${this.animationSpeed}`);
        }
        
        this.animateStep();
    }
    
    /**
     * Pause the animation
     */
    pauseAnimation() {
        this.isAnimating = false;
    }
    
    /**
     * Reset the animation to the beginning
     */
    resetAnimation() {
        this.animationStep = 0;
        this.resetGraphState();
        this.updateProgressChart();
    }
    
    /**
     * Animate a single step of the algorithm
     */
    animateStep() {
        if (!this.isAnimating || this.animationStep >= this.animationSteps.length) {
            this.isAnimating = false;
            return;
        }
        
        const step = this.animationSteps[this.animationStep];
        this.applyAnimationStep(step);
        
        this.animationStep++;
        this.updateProgressChart();
        
        // Schedule next step
        setTimeout(() => {
            if (this.isAnimating) {
                this.animateStep();
            }
        }, 1000 / this.animationSpeed);
    }
    
    /**
     * Apply a single animation step to the graph
     * @param {Object} step - The animation step to apply
     */
    applyAnimationStep(step) {
        switch (step.type) {
            case 'visit':
                this.visitNode(step.nodeId, step.distance);
                break;
            case 'frontier':
                this.addToFrontier(step.nodeId);
                break;
            case 'pivot':
                this.markAsPivot(step.nodeId);
                break;
            case 'current':
                this.markAsCurrent(step.nodeId);
                break;
            case 'path':
                this.markPath(step.source, step.target);
                break;
            case 'reset':
                this.resetNodeState(step.nodeId);
                break;
            case 'batch':
                step.actions.forEach(action => this.applyAnimationStep(action));
                break;
        }
    }
    
    /**
     * Mark a node as visited
     * @param {number} nodeId - ID of the node to visit
     * @param {number} distance - Distance to the node
     */
    visitNode(nodeId, distance) {
        const node = this.graph.nodes.find(n => n.id === nodeId);
        if (node) {
            node.isVisited = true;
            node.isFrontier = false;
            node.isCurrent = false;
            node.distance = distance;
            this.updateNodeColors();
        }
    }
    
    /**
     * Add a node to the frontier
     * @param {number} nodeId - ID of the node to add to frontier
     */
    addToFrontier(nodeId) {
        const node = this.graph.nodes.find(n => n.id === nodeId);
        if (node) {
            node.isFrontier = true;
            this.updateNodeColors();
        }
    }
    
    /**
     * Mark a node as a pivot
     * @param {number} nodeId - ID of the node to mark as pivot
     */
    markAsPivot(nodeId) {
        const node = this.graph.nodes.find(n => n.id === nodeId);
        if (node) {
            node.isPivot = true;
            this.updateNodeColors();
        }
    }
    
    /**
     * Mark a node as the current node being processed
     * @param {number} nodeId - ID of the node to mark as current
     */
    markAsCurrent(nodeId) {
        // Reset all current nodes
        this.graph.nodes.forEach(n => n.isCurrent = false);
        
        const node = this.graph.nodes.find(n => n.id === nodeId);
        if (node) {
            node.isCurrent = true;
            this.updateNodeColors();
        }
    }
    
    /**
     * Mark an edge as part of the shortest path
     * @param {number} source - Source node ID
     * @param {number} target - Target node ID
     */
    markPath(source, target) {
        const link = this.graph.links.find(l => 
            (l.source.id === source && l.target.id === target) || 
            (l.source.id === target && l.target.id === source)
        );
        
        if (link) {
            link.isPath = true;
            this.updateNodeColors();
        }
    }
    
    /**
     * Reset a node to its default state
     * @param {number} nodeId - ID of the node to reset
     */
    resetNodeState(nodeId) {
        const node = this.graph.nodes.find(n => n.id === nodeId);
        if (node) {
            node.isVisited = false;
            node.isFrontier = false;
            node.isPivot = false;
            node.isCurrent = false;
            this.updateNodeColors();
        }
    }
    
    /**
     * Highlight the shortest path from source to target
     * @param {number} target - Target node ID
     * @param {Array} predecessors - Array of predecessors
     */
    highlightPath(target, predecessors) {
        if (!this.graph) return;
        
        // Reset all links and nodes
        this.graph.links.forEach(link => {
            link.isPath = false;
            link.color = undefined;
            link.width = 1;
            link.particleWidth = 0;
            link.particleSpeed = 0;
        });
        
        // Trace back the path from target to source
        let current = target;
        let pathLinks = [];
        let pathNodes = new Set();
        
        // Add target node to path
        pathNodes.add(target);
        
        while (current !== this.sourceNode && predecessors[current] !== undefined) {
            const pred = predecessors[current];
            pathNodes.add(pred); // Add predecessor to path nodes
            
            // Find the link between current and predecessor
            const link = this.graph.links.find(l => 
                (l.source.id === pred && l.target.id === current) || 
                (l.source.id === current && l.target.id === pred)
            );
            
            if (link) {
                link.isPath = true;
                pathLinks.push(link);
            }
            current = pred;
        }
        
        // Create a city/maze-like visualization of the path
        if (pathLinks.length > 0) {
            // Make path links more prominent with city-like appearance
            pathLinks.forEach((link, index) => {
                // Gradient color from source (green) to target (red)
                const progress = index / (pathLinks.length - 1 || 1);
                link.color = `rgb(${Math.floor(255 * progress)}, ${Math.floor(255 * (1 - progress))}, 50)`;
                link.width = 5; // Wider links for better visibility
                
                // Add particles flowing along the path
                link.particleWidth = 6;
                link.particleSpeed = 0.02;
                link.particleColor = '#ffff00'; // Yellow particles
            });
            
            // Highlight path nodes with special appearance
            this.graph.nodes.forEach(node => {
                if (pathNodes.has(node.id)) {
                    // Make path nodes larger and more visible
                    node.val = Math.max(node.val || 10, 15);
                    
                    if (node.id === this.sourceNode) {
                        node.color = '#00ff00'; // Source node in green
                        node.val = 20; // Larger source node
                    } else if (node.id === target) {
                        node.color = '#ff0000'; // Target node in red
                        node.val = 20; // Larger target node
                    } else {
                        node.color = '#ffaa00'; // Path nodes in orange
                    }
                }
            });
        }
        
        // For WebGPU, add enhanced city/maze visualization effects
        if (this.supportsWebGPU && pathLinks.length > 0) {
            // Add glowing effect to path links
            pathLinks.forEach(link => {
                link.width = 6;
                link.particleWidth = 8;
                link.particleSpeed = 0.03;
            });
            
            // Force graph to 2D plane for city/maze-like appearance
            this.graph.nodes.forEach(node => {
                if (!pathNodes.has(node.id)) {
                    // Keep non-path nodes at the same height
                    node.fz = 0;
                }
            });
        }
        
        this.updateNodeColors();
    }
    
    /**
     * Update the progress chart showing algorithm execution
     */
    updateProgressChart(data) {
        const progressContainer = document.getElementById('progress-container');
        if (!progressContainer) return;
        
        // Clear previous chart
        while (progressContainer.firstChild) {
            progressContainer.removeChild(progressContainer.firstChild);
        }
        
        if (!this.animationSteps.length) return;
        
        // Create progress chart
        const chart = document.createElement('div');
        chart.className = 'progress-chart';
        chart.style.display = 'flex';
        chart.style.alignItems = 'center';
        chart.style.height = '100%';
        
        // Add step indicators
        const totalSteps = this.animationSteps.length;
        for (let i = 0; i < totalSteps; i++) {
            const step = document.createElement('div');
            step.className = `progress-step ${i < this.animationStep ? 'active' : ''}`;
            step.style.flex = '1';
            step.style.height = '8px';
            step.style.margin = '0 1px';
            
            // Color based on step type
            if (i < this.animationStep) {
                const stepType = this.animationSteps[i].type;
                switch (stepType) {
                    case 'visit':
                        step.style.backgroundColor = this.nodeColors.visited;
                        break;
                    case 'frontier':
                        step.style.backgroundColor = this.nodeColors.frontier;
                        break;
                    case 'pivot':
                        step.style.backgroundColor = this.nodeColors.pivot;
                        break;
                    case 'current':
                        step.style.backgroundColor = this.nodeColors.current;
                        break;
                    case 'path':
                        step.style.backgroundColor = '#e74c3c';
                        break;
                    default:
                        step.style.backgroundColor = '#3498db';
                }
            }
            
            chart.appendChild(step);
        }
        
        progressContainer.appendChild(chart);
        
        // Add current step info
        if (this.animationStep > 0 && this.animationStep <= totalSteps) {
            const currentStep = this.animationSteps[this.animationStep - 1];
            const stepInfo = document.createElement('div');
            stepInfo.className = 'step-info';
            stepInfo.style.marginTop = '10px';
            stepInfo.style.textAlign = 'center';
            
            let infoText = '';
            switch (currentStep.type) {
                case 'visit':
                    infoText = `Visited node ${currentStep.nodeId} with distance ${currentStep.distance.toFixed(2)}`;
                    break;
                case 'frontier':
                    infoText = `Added node ${currentStep.nodeId} to frontier`;
                    break;
                case 'pivot':
                    infoText = `Selected node ${currentStep.nodeId} as pivot`;
                    break;
                case 'current':
                    infoText = `Processing node ${currentStep.nodeId}`;
                    break;
                case 'path':
                    infoText = `Added edge ${currentStep.source} → ${currentStep.target} to path`;
                    break;
            }
            
            stepInfo.textContent = infoText;
            progressContainer.appendChild(stepInfo);
        }
        
        // Add progress percentage
        const progressPercent = document.createElement('div');
        progressPercent.className = 'progress-percent';
        progressPercent.style.marginTop = '5px';
        progressPercent.style.textAlign = 'center';
        progressPercent.textContent = `Step ${this.animationStep} of ${this.animationSteps.length} (${Math.round((this.animationStep / this.animationSteps.length) * 100)}%)`;
        progressContainer.appendChild(progressPercent);
    }
    
    /**
     * Set the animation speed
     * @param {number} speed - Animation speed multiplier
     */
    setAnimationSpeed(speed) {
        this.animationSpeed = speed;
    }
    
    /**
     * Get the current graph data
     * @returns {Object} Current graph data
     */
    getGraphData() {
        return this.graph;
    }
    
    /**
     * Set the visualization theme
     * @param {string} theme - Theme name to set
     */
    setVisualizationTheme(theme) {
        if (this.themes[theme]) {
            this.visualizationTheme = theme;
            console.log(`Setting visualization theme to: ${theme}`);
            
            // If we already have a graph, update its appearance
            if (this.graph) {
                this.updateThemeAppearance();
            }
        }
    }
    
    /**
     * Update the appearance of the graph based on the current theme
     */
    updateThemeAppearance() {
        if (!this.graph || !this.forceGraph) return;
        
        const theme = this.themes[this.visualizationTheme];
        console.log(`Updating appearance with theme: ${this.visualizationTheme}`);
        
        // Update ground color if using WebGPU
        if (this.supportsWebGPU && this.forceGraph.scene) {
            this.forceGraph.scene().traverse(object => {
                if (object instanceof THREE.Mesh && object.geometry instanceof THREE.PlaneGeometry) {
                    object.material.color.set(theme.groundColor);
                }
            });
        }
        
        // Update node and link colors
        this.graph.nodes.forEach(node => {
            if (node.isBuilding) {
                node.color = theme.buildingColor;
            } else {
                node.color = theme.nodeColor;
            }
        });
        
        this.graph.links.forEach(link => {
            if (!link.isPath) {
                link.color = theme.linkColor;
            }
        });
        
        // Update the visualization
        this.forceGraph.graphData(this.graph);
    }
}

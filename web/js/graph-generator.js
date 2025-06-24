/**
 * Graph Generator Module
 * Generates different types of graphs for visualization and algorithm testing
 */
class GraphGenerator {
    /**
     * Generate a scale-free graph using the Barabási-Albert model
     * @param {number} nodeCount - Number of nodes
     * @param {number} edgesPerNode - Number of edges to attach from a new node to existing nodes
     * @returns {Object} Graph with nodes and links
     */
    static generateScaleFreeGraph(nodeCount, edgesPerNode = 3) {
        const nodes = Array.from({ length: nodeCount }, (_, i) => ({
            id: i,
            label: `Node ${i}`
        }));
        
        const links = [];
        
        // Start with a complete graph of m0 nodes (where m0 = edgesPerNode)
        const m0 = Math.min(edgesPerNode, nodeCount);
        for (let i = 0; i < m0; i++) {
            for (let j = i + 1; j < m0; j++) {
                links.push({
                    source: i,
                    target: j,
                    weight: Math.random() * 10 + 1 // Random weight between 1 and 11
                });
            }
        }
        
        // Add remaining nodes using preferential attachment
        for (let i = m0; i < nodeCount; i++) {
            const degrees = new Array(i).fill(0);
            
            // Calculate degree of each node
            for (const link of links) {
                const source = typeof link.source === 'object' ? link.source.id : link.source;
                const target = typeof link.target === 'object' ? link.target.id : link.target;
                degrees[source]++;
                degrees[target]++;
            }
            
            // Create probability distribution based on degrees
            const totalDegree = degrees.reduce((sum, degree) => sum + degree, 0);
            const probabilities = degrees.map(degree => degree / totalDegree);
            
            // Connect new node to existing nodes
            const connected = new Set();
            for (let e = 0; e < Math.min(edgesPerNode, i); e++) {
                let target;
                do {
                    // Select a node based on probability distribution
                    const r = Math.random();
                    let cumulativeProbability = 0;
                    target = 0;
                    
                    for (let j = 0; j < i; j++) {
                        cumulativeProbability += probabilities[j];
                        if (r <= cumulativeProbability) {
                            target = j;
                            break;
                        }
                    }
                } while (connected.has(target));
                
                connected.add(target);
                links.push({
                    source: i,
                    target,
                    weight: Math.random() * 10 + 1 // Random weight between 1 and 11
                });
            }
        }
        
        return { nodes, links };
    }
    
    /**
     * Generate a 3D grid graph
     * @param {number} xSize - Size in X dimension
     * @param {number} ySize - Size in Y dimension
     * @param {number} zSize - Size in Z dimension
     * @returns {Object} Graph with nodes and links
     */
    static generate3DGridGraph(xSize, ySize, zSize) {
        const nodes = [];
        const links = [];
        
        // Create nodes
        let nodeId = 0;
        for (let z = 0; z < zSize; z++) {
            for (let y = 0; y < ySize; y++) {
                for (let x = 0; x < xSize; x++) {
                    nodes.push({
                        id: nodeId,
                        label: `Node ${nodeId}`,
                        x: x / (xSize - 1 || 1) * 100 - 50,
                        y: y / (ySize - 1 || 1) * 100 - 50,
                        z: z / (zSize - 1 || 1) * 100 - 50
                    });
                    nodeId++;
                }
            }
        }
        
        // Create links (6-connected grid)
        for (let z = 0; z < zSize; z++) {
            for (let y = 0; y < ySize; y++) {
                for (let x = 0; x < xSize; x++) {
                    const currentId = x + y * xSize + z * xSize * ySize;
                    
                    // Connect to right neighbor
                    if (x < xSize - 1) {
                        links.push({
                            source: currentId,
                            target: currentId + 1,
                            weight: Math.random() * 5 + 1 // Random weight between 1 and 6
                        });
                    }
                    
                    // Connect to bottom neighbor
                    if (y < ySize - 1) {
                        links.push({
                            source: currentId,
                            target: currentId + xSize,
                            weight: Math.random() * 5 + 1
                        });
                    }
                    
                    // Connect to back neighbor
                    if (z < zSize - 1) {
                        links.push({
                            source: currentId,
                            target: currentId + xSize * ySize,
                            weight: Math.random() * 5 + 1
                        });
                    }
                }
            }
        }
        
        return { nodes, links };
    }
    
    /**
     * Generate a 3D geometric graph
     * @param {number} nodeCount - Number of nodes
     * @param {number} radius - Connection radius
     * @returns {Object} Graph with nodes and links
     */
    static generate3DGeometricGraph(nodeCount, radius = 0.2) {
        const nodes = Array.from({ length: nodeCount }, (_, i) => ({
            id: i,
            label: `Node ${i}`,
            x: Math.random() * 100 - 50,
            y: Math.random() * 100 - 50,
            z: Math.random() * 100 - 50
        }));
        
        const links = [];
        
        // Connect nodes that are within radius of each other
        for (let i = 0; i < nodeCount; i++) {
            for (let j = i + 1; j < nodeCount; j++) {
                const dx = nodes[i].x - nodes[j].x;
                const dy = nodes[i].y - nodes[j].y;
                const dz = nodes[i].z - nodes[j].z;
                const distance = Math.sqrt(dx * dx + dy * dy + dz * dz);
                
                if (distance < radius * 100) {
                    links.push({
                        source: i,
                        target: j,
                        weight: distance / 10 // Weight proportional to distance
                    });
                }
            }
        }
        
        return { nodes, links };
    }
    
    /**
     * Parse a graph from JSON format
     * @param {string} jsonString - JSON string representing the graph
     * @returns {Object} Parsed graph with nodes and links
     */
    static parseGraphFromJSON(jsonString) {
        try {
            const parsedGraph = JSON.parse(jsonString);
            
            // Validate graph structure
            if (!Array.isArray(parsedGraph.nodes) || !Array.isArray(parsedGraph.links)) {
                throw new Error('Invalid graph format: missing nodes or links arrays');
            }
            
            // Ensure all nodes have id
            parsedGraph.nodes = parsedGraph.nodes.map((node, index) => ({
                id: node.id !== undefined ? node.id : index,
                label: node.label || `Node ${index}`,
                ...node
            }));
            
            // Ensure all links have source, target, and weight
            parsedGraph.links = parsedGraph.links.map((link, index) => ({
                source: link.source,
                target: link.target,
                weight: link.weight !== undefined ? link.weight : 1,
                ...link
            }));
            
            return parsedGraph;
        } catch (error) {
            console.error('Error parsing graph:', error);
            return null;
        }
    }
    
    /**
     * Generate a graph based on the specified type and parameters
     * @param {string} type - Type of graph to generate
     * @param {Object} params - Parameters for graph generation
     * @returns {Object} Generated graph
     */
    static generateGraph(type, params = {}) {
        switch (type) {
            case 'scale-free':
                return this.generateScaleFreeGraph(
                    params.nodeCount || 100,
                    params.edgesPerNode || 3
                );
            case 'grid-3d':
                const size = Math.ceil(Math.cbrt(params.nodeCount || 125));
                return this.generate3DGridGraph(size, size, size);
            case 'geometric-3d':
                return this.generate3DGeometricGraph(
                    params.nodeCount || 100,
                    params.radius || 0.2
                );
            default:
                console.error('Unknown graph type:', type);
                return { nodes: [], links: [] };
        }
    }
}

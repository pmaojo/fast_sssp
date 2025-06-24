Tarea 1.1: Añadir fast_base: bool a FastSSSPConfig
Tarea 1.2: Modificar BMSSP::execute para detectar cuando |S| == 1 y work_set <= k²
Tarea 1.3: Implementar bypass directo a 
mini_dijkstra
 sin transformación de grafo
Tarea 1.4: Actualizar SmartSSSP para considerar este modo en la estimación de coste
Semana 2: Auto-tuning de parámetros k y t
Tarea 2.1: Implementar pilot-run para medir tamaño de work_set con parámetros iniciales
Tarea 2.2: Desarrollar heurística adaptativa: k = min{ 2ⁱ | work_set ≤ c · k² }
Tarea 2.3: Implementar caché LRU para almacenar parámetros óptimos por tipo de grafo
Tarea 2.4: Pruebas de rendimiento comparativas con diferentes configuraciones
Sprint 2: Optimización y Seguridad (2 semanas)
Semana 3: Modo sin sweep final y fuzzing
Tarea 3.1: Añadir feature-flag unsafe_no_sweep en Cargo.toml
Tarea 3.2: Implementar condicionales #[cfg(feature="unsafe_no_sweep")] en código
Tarea 3.3: Desarrollar fuzzer para generar grafos aleatorios y validar precisión
Tarea 3.4: Sistema de registro de contra-ejemplos para depuración
Semana 4: Paralelización de relajación por lotes
Tarea 4.1: Implementar Vec<AtomicU64> para distancias con operaciones atómicas
Tarea 4.2: Integrar Rayon para procesamiento paralelo de lotes
Tarea 4.3: Implementar estrategia Disjoint-Write para evitar data-races
Tarea 4.4: Benchmarks comparativos entre versión secuencial y paralela
Sprint 3: Refinamiento de Heurísticas y Benchmarks (2 semanas)
Semana 5: Heurística de transformación hub-split refinada
Tarea 5.1: Implementar heurística if max_deg > max(64, 4*sqrt(m)) { transform(); }
Tarea 5.2: Añadir detección de hubs basada en distribución de grados
Tarea 5.3: Optimizar transformación para actuar solo en vértices críticos
Tarea 5.4: Pruebas comparativas con diferentes umbrales de transformación
Semana 6: Benchmarks con datasets del mundo real
Tarea 6.1: Integrar datasets de DIMACS, SNAP y KONECT
Tarea 6.2: Desarrollar pipeline CI para benchmarks automatizados
Tarea 6.3: Implementar sistema de comparación nightly FastSSSP vs Dijkstra
Tarea 6.4: Dashboard de rendimiento con histórico de mejoras
Sprint 4: Visualización y Documentación (2 semanas)
Semana 7: Visualización espectacular
Tarea 7.1: Desarrollar frontend para visualizar grafos y ejecución de algoritmos
Tarea 7.2: Implementar animación de propagación de fronteras en BMSSP
Tarea 7.3: Crear visualización comparativa de rendimiento entre algoritmos
Tarea 7.4: Dashboard interactivo para ajustar parámetros y ver resultados en tiempo real
Semana 8: Documentación y publicación
Tarea 8.1: Escribir white-paper técnico explicando las optimizaciones
Tarea 8.2: Crear README avanzado con guías de uso y casos de estudio
Tarea 8.3: Documentar API completa con ejemplos
Tarea 8.4: Preparar presentación para conferencia/blog técnico
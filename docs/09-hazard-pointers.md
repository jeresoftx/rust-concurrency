# Hazard Pointers

> **Curso:** rust-concurrency · **Capítulo:** 09 · **Prerrequisitos:** estructuras
> lock-free, CAS loops, ABA, ownership en Rust, `Arc`, `Mutex` y memory ordering
> básico
> **Código:** [`src/hazard_pointers.rs`](../src/hazard_pointers.rs) · **Video:** pendiente
> **Lección en el sitio:** pendiente

## Introducción

Hazard pointers resuelven una pregunta que aparece después de escribir una
estructura lock-free: cuándo es seguro liberar un nodo. Un hilo puede retirar un
nodo de una lista, pero otro hilo tal vez todavía lo leyó y está a punto de
usarlo. Liberarlo demasiado pronto convierte una carrera lógica en memoria
inválida.

Este capítulo enseña protección de nodos, registros por hilo, listas de nodos
retirados, escaneo, reclamación diferida y costo de mantener memoria viva. El
modelo usa identificadores seguros, no punteros crudos, para aislar el protocolo
antes de discutir `unsafe`.

## Motivación

Una pila lock-free puede retirar su head con un CAS exitoso. Desde el punto de
vista de la pila, el nodo ya no está publicado. Pero otro hilo pudo leer ese
head justo antes del CAS y todavía necesita inspeccionarlo. Si el primer hilo lo
libera de inmediato, el segundo podría tocar memoria ya liberada.

Hazard pointers cambian el contrato: antes de leer un nodo, un hilo publica "yo
estoy protegiendo este nodo". Quien retira nodos no los libera de inmediato; los
pone en una lista de retirados. De vez en cuando escanea todos los hazard
pointers activos. Solo reclama nodos que nadie protege.

## Teoría

### Historia

Los algoritmos lock-free clásicos explican cómo cambiar enlaces con CAS, pero
no siempre explican cómo liberar memoria de forma segura. En lenguajes con
recolección de basura, el recolector puede resolverlo. En Rust, C o C++, el
programador necesita una estrategia explícita.

Hazard pointers son una de esas estrategias. Cada participante tiene uno o más
registros donde publica los nodos que podría tocar. Retirar y reclamar se vuelve
un proceso de dos fases: quitar del algoritmo, y liberar solo después de
comprobar que ningún registro lo protege.

### Fundamentos

Un hazard pointer es una promesa temporal: "este participante puede leer este
nodo". Un nodo retirado ya no pertenece a la estructura pública, pero todavía no
puede liberarse si aparece en algún hazard pointer activo.

El flujo básico es:

```text
proteger nodo
validar que el nodo sigue siendo alcanzable
usar nodo
limpiar protección
retirar nodo cuando sale de la estructura
escanear protecciones
reclamar solo nodos no protegidos
```

El escaneo compara la lista de retirados contra el conjunto de nodos protegidos.
Si un nodo retirado está protegido, se retrasa. Si no está protegido, se reclama.
Un umbral de retiro evita escanear en cada operación y agrupa el costo.

La invariante central del capítulo es:

```text
un nodo retirado solo puede reclamarse cuando ningún participante lo publica
como protegido
```

### Casos de uso

Hazard pointers aparecen en:

- pilas y colas lock-free con nodos en memoria dinámica;
- listas enlazadas concurrentes;
- mapas concurrentes de bajo nivel;
- sistemas sin recolección de basura;
- motores de base de datos y runtimes de ejecución;
- estructuras donde filtrar memoria no es aceptable.

### Ventajas y limitaciones

Ventajas:

- Reclamación explícita y local por participante.
- No requiere pausar a todos los hilos.
- Evita liberar nodos todavía observables.
- Permite razonar sobre memoria en estructuras lock-free.

Limitaciones:

- Cada lectura protegida tiene costo de publicación.
- El escaneo cuesta O(retirados + participantes).
- Un participante que no limpia su hazard pointer retrasa reclamación.
- Requiere disciplina estricta: proteger, validar, usar, limpiar.
- En implementaciones reales suele necesitar `unsafe` y revisión cuidadosa.

### Comparación con alternativas

Epoch GC agrupa reclamación por épocas. Suele ser eficiente cuando los hilos
entran y salen de secciones críticas cortas, pero un participante estancado puede
retener mucha memoria.

El conteo de referencias es más directo conceptualmente, pero cada clon/drop
puede costar operaciones atómicas y puede introducir ciclos si se modela
propiedad compartida.

Un recolector de basura libera al programador de esta contabilidad, pero cambia
el entorno de ejecución y no existe como mecanismo general de Rust.

Filtrar memoria evita use-after-free, pero solo cambia un bug de seguridad por
un bug de retención. Puede ser aceptable en procesos cortos, no en sistemas
vivos.

## Diagramas

El diagrama principal vive en
[`diagrams/09-hazard-pointers.mmd`](../diagrams/09-hazard-pointers.mmd). Muestra
un participante protegiendo un nodo, otro retirándolo, el escaneo de hazards y
la decisión de retrasar o reclamar.

## Análisis de complejidad

| Operación | Mejor caso | Caso promedio | Peor caso | Espacio |
|-----------|------------|---------------|-----------|---------|
| `ParticipantId::new` | O(1) | O(1) | O(1) | O(1) |
| `NodeId::new` | O(1) | O(1) | O(1) | O(1) |
| `RetiredNode::new` | O(1) | O(tamaño de carga) | O(tamaño de carga) | O(tamaño de carga) |
| `HazardDomain::new` | O(1) | O(1) | O(1) | O(1) |
| `HazardDomain::protect` | O(log p) | O(log p) | O(log p) | O(1) adicional |
| `HazardDomain::clear` | O(log p) | O(log p) | O(log p) | O(1) |
| `HazardDomain::is_protected` | O(p) | O(p) | O(p) | O(1) |
| `HazardDomain::protected_nodes` | O(p log p) | O(p log p) | O(p log p) | O(p) |
| `HazardDomain::retire` | O(1) | O(1), o escaneo por umbral | O(r + p log p) | O(1) o O(r) |
| `HazardDomain::scan` | O(r + p log p) | O(r + p log p) | O(r + p log p) | O(r + p) |
| `HazardDomain::retired_nodes` | O(r) | O(r) | O(r) | O(r) |
| `HazardDomain::reclaimed_nodes` | O(n) | O(n) | O(n) | O(n) |

`p` es el número de participantes, `r` el número de nodos retirados y `n` el
historial de nodos reclamados.

## Visualización interactiva (opcional)

No aplica todavía. Una visualización futura debería permitir marcar nodos como
protegidos, retirarlos y ver cómo el escaneo decide entre retrasar y reclamar.

## Implementación

La implementación del curso define:

- `ParticipantId`: registro lógico de un hilo o participante;
- `NodeId`: identificador seguro de nodo;
- `RetiredNode`: nodo retirado con carga descriptiva;
- `ScanReport`: resultado de un escaneo;
- `HazardDomain`: dominio con hazards activos, retirados y reclamados.

La API evita `unsafe`. En una estructura real, `NodeId` sería un puntero o una
referencia a memoria dinámica; aquí es un identificador para enseñar el protocolo
sin exponer al estudiante a uso de memoria inválida.

## Pruebas

Las pruebas cubren:

- protección visible para escaneos;
- reclamación de nodos retirados no protegidos;
- retraso de reclamación mientras un nodo sigue protegido;
- reclamación después de limpiar protección;
- escaneo automático por umbral;
- independencia entre registros por participante.

## Benchmarks

El benchmark manual vive en
[`benches/hazard_pointers_bench.rs`](../benches/hazard_pointers_bench.rs). Mide
protección/limpieza, costo de escaneo con nodos protegidos y reclamación por
umbral.

El objetivo es observar el costo del protocolo. No demuestra que hazard pointers
sean la mejor estrategia; esa decisión depende del patrón de lectura, número de
participantes, memoria retenida y latencia tolerable.

## Ejercicios

### Ejercicio 1: Proteger un nodo `[Nivel 1]`

Crea un dominio, un participante y un nodo. Protege el nodo y verifica que
aparece en `protected_nodes`.

**Entrada/Salida esperada:** `is_protected(node)` devuelve `true`.

<details>
<summary>Pista</summary>
Un participante tiene un registro activo dentro del dominio.
</details>

### Ejercicio 2: Retrasar reclamación `[Nivel 2]`

Protege un nodo, retíralo y ejecuta `scan`.

**Entrada/Salida esperada:** el nodo aparece en `delayed`, no en `reclaimed`.

<details>
<summary>Pista</summary>
Mientras el nodo está protegido, el escaneo debe conservarlo en retirados.
</details>

### Ejercicio 3: Escaneo por umbral `[Nivel 3]`

Crea un dominio con umbral 2, retira dos nodos sin proteger y observa el reporte
automático del segundo retiro.

**Entrada/Salida esperada:** ambos nodos se reclaman en el reporte.

<details>
<summary>Pista</summary>
El primer `retire` devuelve `None`; el segundo alcanza el umbral.
</details>

### Ejercicio 4: Elegir reclamación `[Nivel 4]`

Compara hazard pointers, epoch GC y conteo de referencias para una cola lock-free
con lectores largos ocasionales. Explica qué métrica usarías para elegir.

<details>
<summary>Pista</summary>
Incluye memoria retenida, costo por lectura, latencia de reclamación y facilidad
de auditoría.
</details>

## Soluciones

Las soluciones ejecutables de niveles 1 a 3 viven en:

- [`examples/soluciones/hazard_protect_node.rs`](../examples/soluciones/hazard_protect_node.rs)
- [`examples/soluciones/hazard_delayed_reclaim.rs`](../examples/soluciones/hazard_delayed_reclaim.rs)
- [`examples/soluciones/hazard_threshold_scan.rs`](../examples/soluciones/hazard_threshold_scan.rs)

Para el nivel 4, una respuesta sana empieza por describir la duración esperada
de las lecturas. Si los lectores pueden quedarse estancados, cualquier esquema
que espere quiescencia global puede retener demasiada memoria.

## Referencias

- Maged M. Michael, *Hazard Pointers: Safe Memory Reclamation for Lock-Free Objects*.
- Maurice Herlihy y Nir Shavit, *The Art of Multiprocessor Programming*.
- Mara Bos, *Rust Atomics and Locks*.
- Rust Standard Library: `std::sync::atomic`.

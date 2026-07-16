# Lock-Free Structures

> **Curso:** rust-concurrency · **Capítulo:** 08 · **Prerrequisitos:** atómicos,
> memory ordering, CAS loops, `Arc`, threads y límites de `unsafe`
> **Código:** [`src/lock_free.rs`](../src/lock_free.rs) · **Video:** pendiente
> **Lección en el sitio:** pendiente

## Introducción

Una estructura lock-free no depende de que un hilo específico libere un lock
para que el sistema como conjunto avance. Si un hilo pierde una carrera de CAS,
reintenta; mientras haya contención, alguien debe estar progresando.

Este capítulo enseña garantías de progreso, CAS loops, reintentos, ABA,
compartición falsa y la diferencia entre lock-free y wait-free. El modelo del
curso usa Rust seguro: una pila acotada de `usize` con nodos preasignados e
índices atómicos. Eso permite estudiar la mecánica sin introducir todavía
punteros crudos ni reclamación de memoria.

## Motivación

Un `Mutex<Vec<T>>` es claro y muchas veces suficiente. Pero bajo alta contención,
todos los hilos esperan por una sección crítica única. Una estructura lock-free
intenta cambiar esa espera por competencia atómica: cada hilo propone un cambio
con `compare_exchange`; si otro gana, recarga el estado y vuelve a intentar.

La promesa no es magia. Lock-free no significa rápido siempre, ni significa que
cada hilo terminará pronto. Significa que el sistema no queda detenido porque un
hilo se durmió sosteniendo un lock. Esa garantía cuesta complejidad: ordering,
ABA, reintentos y pruebas más difíciles.

## Teoría

### Historia

Las estructuras lock-free nacen de la necesidad de escalar en sistemas donde el
bloqueo tradicional crea cuellos de botella o riesgos de prioridad. Pilas,
colas y contadores pueden diseñarse con operaciones atómicas, especialmente con
CAS: "si el valor sigue siendo el que observé, cámbialo por este otro".

El algoritmo de Treiber para pilas es uno de los ejemplos clásicos. Lee el head,
prepara el siguiente nodo y hace CAS sobre el head. Si falla, repite. En Rust
real, una pila lock-free general requiere resolver memoria: quién puede liberar
un nodo mientras otro hilo aún lo está leyendo. Ese tema se separa en Hazard
Pointers y Epoch GC.

### Fundamentos

Las garantías de progreso más comunes son:

- blocking: un hilo puede impedir a otros al sostener un lock;
- lock-free: siempre progresa algún hilo del sistema;
- wait-free: cada operación termina en un número acotado de pasos.

Lock-free es más fuerte que blocking, pero más débil que wait-free. Un hilo
individual puede reintentar muchas veces si otros ganan la carrera.

Un CAS loop tiene esta forma:

```text
leer estado actual
calcular nuevo estado
intentar compare_exchange
si falla, observar el nuevo estado y reintentar
```

El problema ABA aparece cuando un hilo observa A, otro cambia A -> B -> A, y el
primer hilo cree que nada cambió porque el valor volvió a ser A. Con punteros,
eso puede ser gravísimo: el mismo valor de dirección puede ocultar una historia
distinta de propiedad y memoria.

La compartición falsa ocurre cuando datos independientes comparten una línea de
caché. Aunque cada hilo toque una variable distinta, el hardware invalida la
misma línea entre núcleos y el rendimiento cae. No cambia la correctitud, pero
sí la forma de medir.

La invariante central del capítulo es:

```text
un CAS exitoso publica una transición de estado; un CAS fallido no es un error,
es evidencia de que otro hilo cambió el estado primero
```

### Casos de uso

Las estructuras lock-free aparecen en:

- colas de alta contención;
- work stealing y planificadores;
- contadores y agregadores de métricas;
- runtimes async;
- bases de datos y motores de almacenamiento;
- sistemas de baja latencia donde bloquear es demasiado caro.

### Ventajas y limitaciones

Ventajas:

- Evitan depender de que un hilo libere un lock.
- Pueden reducir latencia bajo ciertos patrones de contención.
- Hacen explícita la transición atómica de estado.
- Son base conceptual para estructuras concurrentes avanzadas.

Limitaciones:

- Son difíciles de diseñar, probar y revisar.
- Lock-free no implica wait-free.
- ABA puede invalidar razonamientos aparentemente correctos.
- La reclamación de memoria es un problema central, no un detalle.
- Benchmarks pequeños pueden mentir por caché, scheduling y tamaño de mensaje.

### Comparación con alternativas

`Mutex` es mejor cuando la invariante es compuesta y la claridad importa más que
evitar bloqueo. Un lock sencillo suele ganar en mantenibilidad.

Channels son mejores cuando puedes mover propiedad de unidades de trabajo en vez
de compartir una estructura concurrente.

Sharding divide la contención en varios locks o estructuras. Muchas veces da
gran parte del beneficio con menos complejidad que lock-free.

Wait-free ofrece una garantía más fuerte, pero suele requerir algoritmos mucho
más especializados y costos adicionales.

## Diagramas

El diagrama principal vive en
[`diagrams/08-lock-free.mmd`](../diagrams/08-lock-free.mmd). Muestra un CAS loop
de pila: leer head, preparar transición, intentar CAS, reintentar si otro hilo
ganó y registrar el riesgo ABA.

## Análisis de complejidad

| Operación | Mejor caso | Caso promedio | Peor caso | Espacio |
|-----------|------------|---------------|-----------|---------|
| `ProgressGuarantee::stronger_than` | O(1) | O(1) | O(1) | O(1) |
| `BoundedLockFreeStack::new` | O(n) | O(n) | O(n) | O(n) |
| `BoundedLockFreeStack::capacity` | O(1) | O(1) | O(1) | O(1) |
| `BoundedLockFreeStack::push` | O(1) | O(1) esperado | reintentos no acotados para un hilo | O(1) |
| `BoundedLockFreeStack::pop` | O(1) | O(1) esperado | reintentos no acotados para un hilo | O(1) |
| `BoundedLockFreeStack::observations` | O(1) | O(1) | O(1) | O(1) |
| `AbaScenario::new` | O(1) | O(1) | O(1) | O(1) |
| `AbaScenario::is_aba_risk` | O(1) | O(1) | O(1) | O(1) |
| `AbaScenario::description` | O(1) | O(1) | O(1) | O(1) |

La pila es acotada: no reserva memoria durante `push` o `pop`. Eso simplifica el
modelo y evita `unsafe`, pero también cambia el contrato: `push` puede fallar
con `StackError::Full`.

## Visualización interactiva (opcional)

No aplica todavía. Una visualización futura debería mostrar varios hilos leyendo
el mismo head, un CAS ganador, CAS fallidos y el patrón A -> B -> A del problema
ABA.

## Implementación

La implementación del curso define:

- `ProgressGuarantee`: comparación entre blocking, lock-free y wait-free;
- `CasObservation`: intentos CAS, éxitos y reintentos;
- `BoundedLockFreeStack`: pila lock-free acotada de `usize`;
- `StackError`: error de capacidad llena;
- `AbaScenario`: descripción segura del patrón ABA.

La pila usa dos listas enlazadas por índices atómicos: `head` para la pila y
`free` para nodos disponibles. `push` toma un nodo de `free`, escribe el valor y
lo publica en `head`. `pop` retira un nodo de `head`, lee el valor y lo devuelve
a `free`.

No se usa `unsafe`. Esto es deliberado: el capítulo enseña la forma del
algoritmo y deja punteros, lifetimes de nodos retirados y reclamación real para
Hazard Pointers y Epoch GC.

## Pruebas

Las pruebas cubren:

- semántica LIFO;
- error por capacidad llena;
- garantía de progreso declarada;
- observaciones CAS;
- push concurrente y pop posterior sin pérdida de valores;
- escenario ABA modelado de forma segura.

## Benchmarks

El benchmark manual vive en
[`benches/lock_free_bench.rs`](../benches/lock_free_bench.rs). Compara la pila
lock-free acotada contra un `Mutex<Vec<usize>>` en carga secuencial y concurrente.

La medición es educativa, no una sentencia universal. Una estructura lock-free
puede perder contra un lock sencillo en cargas pequeñas, y aun así ser valiosa
por su garantía de progreso bajo ciertos fallos y patrones de contención.

## Ejercicios

### Ejercicio 1: Pila LIFO `[Nivel 1]`

Crea una `BoundedLockFreeStack`, inserta tres valores y extráelos.

**Entrada/Salida esperada:** los valores salen en orden inverso al de entrada.

<details>
<summary>Pista</summary>
Una pila publica cada nuevo nodo como `head`.
</details>

### Ejercicio 2: ABA `[Nivel 2]`

Crea un `AbaScenario` donde el head cambia de `3` a `1` y regresa a `3`.

**Entrada/Salida esperada:** `is_aba_risk()` devuelve `true`.

<details>
<summary>Pista</summary>
ABA significa que el valor inicial y final son iguales, pero hubo un valor
intermedio distinto.
</details>

### Ejercicio 3: Push concurrente `[Nivel 3]`

Comparte una pila con `Arc`, lanza varios hilos que hagan `push` y después
extrae todos los valores.

**Entrada/Salida esperada:** no se pierde ningún valor insertado.

<details>
<summary>Pista</summary>
Usa capacidad suficiente para que el ejercicio no falle por `Full`.
</details>

### Ejercicio 4: Elegir una estrategia `[Nivel 4]`

Diseña una cola para un sistema de ingestión con alta contención. Compara tres
opciones: `Mutex<VecDeque<T>>`, sharding con varios locks y cola lock-free.
Explica qué medirías antes de elegir.

<details>
<summary>Pista</summary>
Incluye latencia de cola, rendimiento, saturación de CPU, memoria retenida y
costo de depurar incidentes.
</details>

## Soluciones

Las soluciones ejecutables de niveles 1 a 3 viven en:

- [`examples/soluciones/lock_free_lifo_stack.rs`](../examples/soluciones/lock_free_lifo_stack.rs)
- [`examples/soluciones/lock_free_aba_scenario.rs`](../examples/soluciones/lock_free_aba_scenario.rs)
- [`examples/soluciones/lock_free_concurrent_stack.rs`](../examples/soluciones/lock_free_concurrent_stack.rs)

Para el nivel 4, una respuesta sana no elige lock-free por prestigio. Primero
define la invariante, la carga esperada, el costo de bloqueo, el costo de perder
claridad y qué historia de recuperación de memoria necesita el diseño.

## Referencias

- R. Kent Treiber, *Systems Programming: Coping with Parallelism*.
- Maurice Herlihy y Nir Shavit, *The Art of Multiprocessor Programming*.
- Rust Standard Library: `std::sync::atomic`.
- Mara Bos, *Rust Atomics and Locks*.

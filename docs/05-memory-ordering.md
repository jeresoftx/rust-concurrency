# Memory Ordering

> **Curso:** rust-concurrency · **Capítulo:** 05 · **Prerrequisitos:** atómicos, `Arc`, threads básicos y lectura de `std::sync::atomic::Ordering`
> **Código:** [`src/memory_ordering.rs`](../src/memory_ordering.rs) · **Video:** pendiente
> **Lección en el sitio:** pendiente

## Introducción

La atomicidad evita data races sobre una celda. Memory ordering responde una
pregunta más difícil: qué puede asumir un hilo sobre el orden en que observa
otros datos. Dos operaciones pueden ser atómicas y aun así no publicar una
invariante completa entre hilos.

Este capítulo enseña happens-before, reordenamiento de compilador y CPU,
`Relaxed`, `Acquire`, `Release`, `AcqRel` y `SeqCst`. La meta no es memorizar una
tabla. La meta es aprender a elegir el ordering más débil que todavía expresa la
garantía que el programa necesita.

## Motivación

Supón que un hilo escribe un payload y luego marca una bandera `ready = true`.
Otro hilo espera la bandera y después lee el payload. Si la bandera se publica
con `Release` y se lee con `Acquire`, el lector que observa `true` también puede
razonar sobre las escrituras previas del escritor. Esa relación es
happens-before.

Ahora supón que solo quieres contar eventos. Un contador `Relaxed` puede ser
suficiente: necesitas atomicidad de una variable, pero no estás usando el
contador para publicar otros datos. Pedir `SeqCst` ahí quizá no mejora la
correctitud; solo hace más fuerte una garantía que no estabas usando.

## Teoría

### Historia

Los procesadores y compiladores reordenan operaciones para ejecutar más rápido,
siempre que respeten las reglas del lenguaje y del hardware. En código
monohilo, muchos reordenamientos son invisibles. En concurrencia, el orden en
que un hilo publica y otro observa datos se vuelve parte del contrato.

Los modelos de memoria existen para darle reglas a ese contrato. Rust adopta el
modelo de atómicos de C++ en `std::sync::atomic`, con orderings explícitos. Esta
decisión obliga al programador a escribir qué garantía necesita.

### Fundamentos

`Ordering::Relaxed` garantiza atomicidad de la variable, pero no sincroniza otros
datos. Es útil para contadores y estadísticas donde no se publica una invariante
compuesta.

`Ordering::Release` se usa en una escritura que publica trabajo previo.
`Ordering::Acquire` se usa en una lectura que consume esa publicación. Cuando
una lectura acquire observa una escritura release compatible, se establece una
relación happens-before.

`Ordering::AcqRel` combina acquire y release en operaciones read-modify-write,
como `compare_exchange` o `fetch_add` cuando la operación lee y publica.

`Ordering::SeqCst` agrega una restricción fuerte: todas las operaciones
secuencialmente consistentes participan en un orden global único. Es más fácil
de razonar, pero no debe usarse como sustituto de entender la invariante.

La invariante central del capítulo es:

```text
un hilo solo puede depender de escrituras de otro hilo cuando existe una
relación happens-before que las conecta
```

### Casos de uso

Memory ordering importa en:

- publicación de datos con bandera `ready`;
- colas lock-free;
- CAS loops que publican o consumen estado;
- contadores relajados de métricas;
- algoritmos donde un dato atómico protege acceso a otro dato;
- estructuras concurrentes de bajo nivel.

No todo código concurrente debe manipular orderings directamente. `Mutex`,
`RwLock`, channels y otros tipos seguros ya encapsulan esas garantías.

### Ventajas y limitaciones

Ventajas:

- Permite expresar garantías precisas sin locks.
- Evita pagar por orden global cuando no se necesita.
- Hace explícita la diferencia entre atomicidad y publicación.
- Permite implementar estructuras concurrentes de bajo nivel.

Limitaciones:

- Es fácil elegir un ordering demasiado débil y crear bugs raros.
- Es fácil elegir `SeqCst` por miedo y ocultar falta de razonamiento.
- Las pruebas no pueden demostrar todos los interleavings posibles.
- Los benchmarks dependen mucho de arquitectura, compilador y carga.
- El modelo es conceptual: medir no reemplaza probar la invariante.

### Comparación con alternativas

Un `Mutex` o `RwLock` suele ser mejor cuando la invariante involucra varios
campos o cuando el equipo necesita claridad antes que microcontrol. Un channel
puede expresar ownership transferido sin exponer orderings. `Relaxed` es mejor
para contadores que no publican otros datos. `Acquire`/`Release` son naturales
para publicar y consumir. `SeqCst` es útil cuando se necesita una historia global
sencilla o cuando todavía se está estabilizando un diseño.

## Diagramas

El diagrama principal vive en
[`diagrams/05-memory-ordering.mmd`](../diagrams/05-memory-ordering.mmd). Muestra
un escritor que publica payload con `Release`, un lector que consume con
`Acquire`, y la separación entre contador relaxed y CAS con orderings de éxito y
fallo.

## Análisis de complejidad

| Operación | Mejor caso | Caso promedio | Peor caso | Espacio |
|-----------|------------|---------------|-----------|---------|
| `describe_ordering` | O(1) | O(1) | O(1) | O(1) |
| `PublishedValue::new` | O(1) | O(1) | O(1) | O(1) |
| `PublishedValue::publish` | O(1) | O(1) | O(1) | O(1) |
| `PublishedValue::try_read` | O(1) | O(1) | O(1) | O(1) |
| `RelaxedCounter::increment` | O(1) | O(1) | O(1), con costo de contención | O(1) |
| `RelaxedCounter::load` | O(1) | O(1) | O(1) | O(1) |
| `OrderingCasCell::compare_exchange` | O(1) por intento | O(1) | O(1) por intento | O(1) |
| `OrderingCasCell::observation` | O(1) | O(1) | O(1) | O(1) |

La complejidad algorítmica no captura toda la diferencia. Dos orderings pueden
ser O(1), pero tener costos distintos según arquitectura y barreras necesarias.

## Visualización interactiva (opcional)

No aplica todavía. Una visualización futura debería mostrar líneas de tiempo:
stores del escritor, loads del lector y cuándo existe o no existe
happens-before.

## Implementación

La implementación del curso define:

- `OrderingGuarantee`: resumen educativo de garantías;
- `describe_ordering`: comparación de orderings;
- `PublishedValue`: publicación release/acquire;
- `RelaxedCounter`: contador que no promete publicación;
- `OrderingCasCell`: CAS con orderings explícitos de éxito y fallo.

El modelo evita `unsafe`. En un curso posterior, estructuras lock-free y
reclamación de memoria necesitarán razonar con más cuidado sobre datos no
atómicos, punteros y vida de memoria.

## Pruebas

Las pruebas cubren:

- publicación con `Release` y lectura con `Acquire`;
- lectura antes de publicar;
- contador relaxed con agregación concurrente;
- CAS con orderings explícitos de éxito y fallo;
- descripción de garantías para `Relaxed`, `Acquire`, `Release` y `SeqCst`.

## Benchmarks

El benchmark manual vive en
[`benches/memory_ordering_bench.rs`](../benches/memory_ordering_bench.rs). Compara
`fetch_add` y `load` con algunos orderings.

Los resultados son una pista, no una ley. El rendimiento depende de arquitectura,
compilador, versión de Rust, número de hilos y contención. El benchmark existe
para enseñar que las garantías tienen costo potencial, no para elegir orderings
por cronómetro sin entender la invariante.

## Ejercicios

### Ejercicio 1: Publicación `[Nivel 1]`

Crea un `PublishedValue`, verifica que `try_read` devuelve `None`, publica un
valor y vuelve a leer.

**Entrada/Salida esperada:** `None` antes de publicar, `Some(valor)` después.

<details>
<summary>Pista</summary>
La bandera se escribe con `Release` y se lee con `Acquire`.
</details>

### Ejercicio 2: Contador relaxed `[Nivel 2]`

Lanza varios hilos que incrementen un `RelaxedCounter`.

**Entrada/Salida esperada:** el total final coincide con la suma de incrementos.

<details>
<summary>Pista</summary>
`Relaxed` conserva atomicidad del contador, no publicación de otros datos.
</details>

### Ejercicio 3: CAS con orderings `[Nivel 3]`

Usa `OrderingCasCell` para hacer un CAS exitoso y uno fallido. Observa los
orderings registrados.

**Entrada/Salida esperada:** un éxito, un fallo y valor final igual al primer
intercambio.

<details>
<summary>Pista</summary>
El ordering de fallo no puede ser `Release` ni `AcqRel` en la API estándar.
</details>

### Ejercicio 4: Diseñar un protocolo `[Nivel 4]`

Diseña un protocolo de publicación para dos campos de configuración y una
bandera `ready`. Explica por qué el lector no debe confiar en los campos si no
observó la bandera con acquire.

<details>
<summary>Pista</summary>
Primero define qué escritura release sincroniza con qué lectura acquire.
</details>

## Soluciones

Las soluciones ejecutables de niveles 1 a 3 viven en:

- [`examples/soluciones/memory_ordering_publish.rs`](../examples/soluciones/memory_ordering_publish.rs)
- [`examples/soluciones/memory_ordering_relaxed_counter.rs`](../examples/soluciones/memory_ordering_relaxed_counter.rs)
- [`examples/soluciones/memory_ordering_cas.rs`](../examples/soluciones/memory_ordering_cas.rs)

Para el nivel 4, una respuesta sana empieza por dibujar la relación
happens-before. Si no puedes señalar qué release publica y qué acquire consume,
el protocolo todavía no está listo para implementarse con atómicos.

## Referencias

- Rust Standard Library: `std::sync::atomic::Ordering`.
- Rustonomicon: atomics y memory ordering.
- Mara Bos, *Rust Atomics and Locks*.
- Hans-J. Boehm y Sarita V. Adve, trabajos sobre modelos de memoria.

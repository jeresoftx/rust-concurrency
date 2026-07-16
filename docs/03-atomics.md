# Atomics

> **Curso:** rust-concurrency · **Capítulo:** 03 · **Prerrequisitos:** `Mutex`, `RwLock`, ownership, `Arc`, enteros primitivos y threads básicos
> **Código:** [`src/atomics.rs`](../src/atomics.rs) · **Video:** pendiente
> **Lección en el sitio:** pendiente

## Introducción

Una operación atómica ocurre como una unidad indivisible para una variable
compartida. Si varios hilos incrementan un contador atómico, cada incremento ve
un valor coherente de esa celda y no pisa la actualización de otro hilo. Esta es
una herramienta potente, pero estrecha: protege una operación sobre una variable,
no una invariante arbitraria de dominio.

Este capítulo enseña `load`, `store`, `fetch_add`, `compare_exchange`, contadores,
banderas, loops CAS y una política explícita de overflow. El objetivo no es
sustituir locks por reflejo. El objetivo es saber cuándo una operación atómica
expresa toda la invariante y cuándo solo disfraza un diseño incompleto.

## Motivación

Imagina un servidor que registra cuántas peticiones ha procesado. Cada worker
solo necesita sumar `1`. Un `Mutex<usize>` funciona, pero obliga a todos los
hilos a formar fila para una operación que el hardware puede expresar como una
actualización atómica simple.

Ahora cambia el problema: además del contador, necesitas actualizar un mapa, una
marca de tiempo y una lista de errores como una sola decisión. Un atómico ya no
alcanza. La frontera importante es esta: los atómicos brillan cuando la
invariante cabe en una operación pequeña; los locks siguen siendo honestos
cuando la invariante cruza varios datos.

## Teoría

### Historia

Las instrucciones atómicas aparecen cerca del hardware: test-and-set,
fetch-and-add, compare-and-swap y variantes modernas permiten coordinar hilos
sin entrar a un lock del sistema operativo para cada operación. Sobre esas
instrucciones se construyen contadores, flags, colas lock-free y algoritmos de
sincronización más complejos.

Rust expone estas ideas en `std::sync::atomic`. El lenguaje obliga a elegir un
`Ordering`, porque la atomicidad de una variable y la visibilidad ordenada entre
varias variables son conceptos distintos. En este capítulo usamos
`Ordering::Relaxed` para concentrarnos en operaciones atómicas individuales. El
capítulo de memory ordering estudia `Acquire`, `Release`, `AcqRel` y `SeqCst`.

### Fundamentos

Una operación atómica garantiza que no hay data race sobre esa celda. Las
operaciones básicas son:

- `load`: leer el valor actual;
- `store`: escribir un valor nuevo;
- `fetch_add`: sumar y obtener el valor anterior;
- `compare_exchange`: cambiar el valor solo si todavía coincide con lo esperado.

`compare_exchange` devuelve éxito o fallo porque otro hilo pudo modificar la
celda entre tu lectura y tu intento de escritura. Un CAS loop repite ese patrón:
lee, calcula, intenta cambiar y reintenta si observó un valor distinto.

La invariante de un contador atómico es pequeña:

```text
cada incremento exitoso contribuye exactamente una vez al valor final
```

La invariante de un máximo atómico también cabe en una celda:

```text
el valor guardado nunca baja y eventualmente refleja el mayor candidato exitoso
```

La política de overflow debe ser explícita. En este curso, `AtomicCounter`
satura en `usize::MAX` para evitar que un contador educativo regrese
silenciosamente a cero.

### Casos de uso

Los atómicos son útiles para:

- contadores de métricas;
- banderas de cancelación o publicación simple;
- generación de identificadores monotónicos;
- estadísticas aproximadas;
- CAS loops sobre una sola palabra;
- coordinación de bajo nivel donde un lock sería demasiado caro o imposible.

No son buena herramienta para proteger estructuras grandes con muchas
invariantes internas. Un atómico no hace que un `HashMap` sea concurrente ni
convierte una actualización de varias variables en una transacción.

### Ventajas y limitaciones

Ventajas:

- No requieren adquirir un lock para operaciones simples.
- Escalan bien para contadores y flags de baja complejidad.
- Permiten construir algoritmos lock-free.
- Exponen fallos de CAS de forma explícita.
- Tienen costo bajo cuando la contención es moderada y la operación es pequeña.

Limitaciones:

- Son fáciles de usar mal si la invariante real no cabe en una variable.
- Requieren entender memory ordering para publicación entre datos distintos.
- Bajo mucha contención, un CAS loop puede girar y desperdiciar CPU.
- No reemplazan diseño de dominio, ownership ni partición de estado.
- Pueden volver ilegible el código si se usan para evitar una abstracción clara.

### Comparación con alternativas

Un `Mutex` es mejor cuando una invariante involucra varias operaciones o varios
campos. Un channel es mejor cuando quieres transferir ownership y modelar un
flujo de trabajo. La agregación local por hilo puede ser mejor que un contador
atómico global si la contención es alta y puedes combinar resultados al final.
El código secuencial sigue siendo la mejor opción cuando no hay concurrencia
real.

Los atómicos no son una medalla de "código avanzado". Son una herramienta
precisa para problemas pequeños y muy frecuentes.

## Diagramas

El diagrama principal vive en [`diagrams/03-atomics.mmd`](../diagrams/03-atomics.mmd).
Muestra tres hilos interactuando con una celda atómica y un CAS loop que reintenta
cuando observa un valor inesperado.

## Análisis de complejidad

| Operación | Mejor caso | Caso promedio | Peor caso | Espacio |
|-----------|------------|---------------|-----------|---------|
| `AtomicCounter::new` | O(1) | O(1) | O(1) | O(1) |
| `AtomicCounter::load` | O(1) | O(1) | O(1) | O(1) |
| `AtomicCounter::store` | O(1) | O(1) | O(1) | O(1) |
| `AtomicCounter::fetch_add` | O(1) | O(1) esperado | reintentos bajo contención | O(1) |
| `AtomicFlag::is_set` | O(1) | O(1) | O(1) | O(1) |
| `AtomicFlag::set` | O(1) | O(1) | O(1) | O(1) |
| `CompareExchange::compare_exchange` | O(1) | O(1) | O(1) por intento | O(1) |
| `AtomicMax::record` | O(1) si no actualiza | O(1) esperado | reintentos bajo contención | O(1) |
| `observations` | O(1) | O(1) | O(1) | O(1) |

La complejidad no revela todo el costo. En contención fuerte, varias operaciones
pueden invalidar la misma línea de caché entre núcleos. Por eso el benchmark
incluye agregación local por hilo: a veces la mejor sincronización es hacer
menos sincronización.

## Visualización interactiva (opcional)

No aplica todavía. Una visualización futura podría animar un CAS loop y mostrar
fallos por interleavings. Por ahora, los tests y el diagrama son suficientes
para el modelo mental del capítulo.

## Implementación

La implementación del curso define cuatro tipos:

- `AtomicCounter`: contador con `load`, `store` y `fetch_add` saturante;
- `AtomicFlag`: bandera booleana atómica;
- `CompareExchange`: celda para estudiar éxito y fallo de CAS;
- `AtomicMax`: registro de máximo construido con CAS loop.

Todos usan `Ordering::Relaxed` porque este capítulo se limita a una variable
atómica por operación. Esa decisión es deliberada: antes de hablar de
publicación entre datos, necesitamos entender atomicidad local.

## Pruebas

Las pruebas cubren:

- `fetch_add` devuelve el valor anterior;
- el contador no pierde actualizaciones concurrentes;
- el overflow satura en `usize::MAX`;
- una bandera puede publicarse y observarse;
- `compare_exchange` reporta éxito y fallo;
- un CAS loop registra el máximo entre varios hilos.

También hay pruebas unitarias para `store` y para observaciones de bandera.

## Benchmarks

El benchmark manual vive en [`benches/atomics_bench.rs`](../benches/atomics_bench.rs).
Compara tres enfoques:

- incrementos atómicos globales;
- incrementos protegidos por `Mutex`;
- agregación local por hilo y combinación final.

La comparación enseña un punto importante: el contador atómico puede evitar un
lock, pero un diseño que reduce coordinación compartida puede ganar todavía más.

## Ejercicios

### Ejercicio 1: Traza de contador `[Nivel 1]`

Crea un `AtomicCounter` con valor inicial `10`, ejecuta `fetch_add(5)` y lee el
valor final.

**Entrada/Salida esperada:** valor anterior `10`, valor final `15`,
`fetch_adds` igual a `1`.

<details>
<summary>Pista</summary>
`fetch_add` devuelve el valor antes de sumar.
</details>

### Ejercicio 2: Bandera publicada `[Nivel 2]`

Usa `Arc<AtomicFlag>` para que un hilo espere hasta que otro publique `true`.

**Entrada/Salida esperada:** el worker termina después de observar la bandera.

<details>
<summary>Pista</summary>
Para este ejemplo educativo puedes usar `thread::yield_now()` dentro del ciclo
de espera.
</details>

### Ejercicio 3: Máximo con CAS `[Nivel 3]`

Lanza varios hilos con candidatos distintos y registra el valor máximo con
`AtomicMax`.

**Entrada/Salida esperada:** el valor final es el mayor candidato.

<details>
<summary>Pista</summary>
El CAS loop solo intenta escribir si el candidato es mayor que el valor actual.
</details>

### Ejercicio 4: Elegir entre atómico y agregación local `[Nivel 4]`

Diseña una medición para comparar un contador atómico global contra contadores
locales por hilo. Explica qué pierde y qué gana cada opción.

<details>
<summary>Pista</summary>
El contador global da visibilidad inmediata; la agregación local reduce
contención, pero solo produce el total al combinar resultados.
</details>

## Soluciones

Las soluciones ejecutables de niveles 1 a 3 viven en:

- [`examples/soluciones/atomics_counter_trace.rs`](../examples/soluciones/atomics_counter_trace.rs)
- [`examples/soluciones/atomics_publish_flag.rs`](../examples/soluciones/atomics_publish_flag.rs)
- [`examples/soluciones/atomics_record_max.rs`](../examples/soluciones/atomics_record_max.rs)

Para el nivel 4 no hay una única respuesta. Si necesitas observar el contador en
tiempo real, el atómico global es simple. Si solo necesitas un total al final,
la agregación local puede reducir contención. Si el conteo forma parte de una
decisión con más estado, probablemente necesitas un lock, un actor o rediseñar
el flujo.

## Referencias

- Rust Standard Library: `std::sync::atomic`.
- Rustonomicon: secciones sobre atomics y memory ordering.
- Mara Bos, *Rust Atomics and Locks*.
- Maurice Herlihy y Nir Shavit, *The Art of Multiprocessor Programming*.

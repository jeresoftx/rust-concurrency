# Mutex

> **Curso:** rust-concurrency · **Capítulo:** 01 · **Prerrequisitos:** Rust básico, ownership, referencias y `thread::spawn`
> **Código:** [`src/mutex.rs`](../src/mutex.rs) · **Video:** pendiente
> **Lección en el sitio:** pendiente

## Introducción

Un mutex protege una región de estado para que solo un hilo pueda acceder a ella
de forma mutable a la vez. El nombre viene de *mutual exclusion*: exclusión
mutua. En Rust, un mutex combina sincronización del sistema con una idea muy
propia del lenguaje: el acceso al dato protegido ocurre a través de un guard que
libera el lock al salir de scope.

Este capítulo abre el curso porque enseña el vocabulario que usaremos en todo lo
demás: estado compartido, sección crítica, invariante protegida, contención,
poisoning y costo de serializar trabajo. Se asume que ya puedes leer `struct`,
`impl`, closures, `Result`, `Arc` a nivel de uso y threads básicos.

## Motivación

Imagina un servicio que procesa reservas en paralelo. Cada worker valida una
petición, calcula disponibilidad y aumenta un contador compartido de reservas
confirmadas. Si dos hilos leen el contador al mismo tiempo, ambos podrían ver
`41`, sumar uno y escribir `42`. El sistema procesó dos reservas, pero el estado
solo refleja una.

Ese bug no es falta de aritmética; es falta de exclusión. La operación "leer,
modificar, escribir" debe ocurrir como una unidad lógica. Un mutex hace que el
estado compartido tenga una puerta: mientras un hilo está dentro de la sección
crítica, los demás esperan o reciben una señal de contención.

## Teoría

### Historia

La exclusión mutua aparece tan pronto como los sistemas permiten ejecución
concurrente sobre memoria compartida. Edsger Dijkstra formuló problemas
fundamentales de sincronización en los años sesenta, incluyendo semáforos y
secciones críticas. El mutex moderno es una especialización práctica: un lock
binario asociado a un recurso que debe ser usado por un actor a la vez.

Rust no inventa el mutex, pero lo vuelve pedagógicamente interesante. En vez de
entregar un puntero desnudo al dato, entrega un `MutexGuard`. Ese guard expresa
en el tipo que el acceso mutable vive dentro de una sección crítica y que liberar
el lock no depende de recordar llamar una función manualmente.

### Fundamentos

Un mutex tiene dos piezas conceptuales:

- un estado de sincronización: libre u ocupado;
- un valor protegido: el dato cuya invariante se cuida.

La invariante central es:

```text
mientras exista acceso mutable al valor protegido, ningún otro hilo puede tener
acceso mutable o compartido no coordinado al mismo valor
```

En Rust, `std::sync::Mutex<T>` devuelve un `MutexGuard<T>`. El guard implementa
`DerefMut`, por lo que permite modificar `T`. Cuando el guard sale de scope, su
`Drop` libera el lock. Esta regla hace que el desbloqueo sea estructural: el
flujo del programa determina cuándo termina la sección crítica.

Poisoning es una señal adicional. Si un hilo entra en pánico mientras sostiene
el lock, Rust marca el mutex como poisoned. Eso no significa que la memoria esté
corrupta; significa que la invariante lógica protegida pudo quedar a medias. El
programa puede recuperar el valor, pero debe hacerlo explícitamente.

### Casos de uso

Los mutexes aparecen en sistemas reales cuando varios hilos comparten estado:

- contadores y métricas internas;
- cachés pequeñas protegidas por un lock;
- colas de trabajo sencillas;
- estructuras administrativas en servidores;
- estado de pruebas concurrentes;
- recursos que no son thread-safe por sí mismos.

El mutex no es una estrategia universal. Es una herramienta clara cuando el
estado compartido es pequeño, la sección crítica es corta y la invariante que se
protege cabe en una explicación simple.

### Ventajas y limitaciones

Ventajas:

- Modelo mental directo: una puerta para una sección crítica.
- Encaja bien con ownership y RAII en Rust.
- Protege invariantes que necesitan varias operaciones sobre el mismo dato.
- Permite recuperar poisoning cuando la aplicación sabe reparar el estado.

Limitaciones:

- Serializa el trabajo dentro de la sección crítica.
- Puede crear contención si demasiados hilos compiten por el mismo lock.
- Puede participar en deadlocks si se combinan varios locks sin orden claro.
- No mejora el diseño si el problema realmente pedía mensajes, partición de
  estado o atomics.
- Mantener el lock durante operaciones lentas bloquea a otros hilos sin aportar
  seguridad extra.

### Comparación con alternativas

Un `AtomicUsize` es mejor para un contador simple donde una operación atómica
expresa toda la invariante. Un `RwLock` puede ser mejor cuando hay muchas
lecturas y pocas escrituras. Un channel evita compartir memoria mutable y modela
mejor pipelines o worker pools. Un actor concentra el estado en un solo dueño y
recibe mensajes.

El mutex es buena elección cuando varias operaciones deben ocurrir juntas sobre
un mismo valor. No es buena elección cuando se usa para esconder un diseño
global mutable que todos los hilos tocan todo el tiempo.

## Diagramas

El diagrama principal vive en [`diagrams/01-mutex.mmd`](../diagrams/01-mutex.mmd).
Muestra tres hilos compitiendo por una sola sección crítica y el momento en que
el guard libera el lock al salir de scope.

## Análisis de complejidad

| Operación | Mejor caso | Caso promedio | Peor caso | Espacio |
|-----------|------------|---------------|-----------|---------|
| `new` | O(1) | O(1) | O(1) | O(1) adicional |
| `lock` | O(1) sin contención | depende del scheduler | espera no acotada si otro hilo no libera | O(1) |
| `with_lock` | O(1) + costo del closure | depende del scheduler y del closure | espera no acotada si otro hilo no libera | O(1) |
| `try_with_lock` | O(1) | O(1) | O(1) | O(1) |
| `recover_with` | O(1) + costo del closure | depende del scheduler y del closure | espera no acotada si otro hilo no libera | O(1) |
| `observations` | O(1) | O(1) | O(1) | O(1) |

La tabla separa complejidad algorítmica de costo de sincronización. Un lock no
recorre `n` elementos, pero puede esperar por decisiones externas: cuánto tarda
otro hilo, cómo planifica el scheduler, si el hilo dueño entra en pánico o si la
sección crítica hace más trabajo del necesario.

`try_with_lock` es importante porque no espera. Puede fallar aunque el programa
esté sano: `None` significa "no entré ahora", no "el dato está mal".

## Visualización interactiva (opcional)

No aplica todavía. Un playground visual de interleavings sería útil más adelante
en `academy-web`, pero este capítulo ya puede enseñar el concepto con pruebas,
diagramas y ejemplos ejecutables.

## Implementación

La implementación del curso usa `EducationalMutex<T>`, un wrapper pedagógico
sobre `std::sync::Mutex<T>`. No intenta reemplazar a la biblioteca estándar ni
implementar primitivas del sistema operativo. Su propósito es hacer observables
las señales que el capítulo discute:

- intentos bloqueantes de lock;
- intentos no bloqueantes;
- contención detectada por `try_with_lock`;
- recuperaciones explícitas después de poisoning.

La decisión clave es que `lock` conserva el comportamiento de la biblioteca
estándar y devuelve el guard. En cambio, `with_lock` mantiene el guard dentro de
un closure para enseñar scope corto. Si ocurre poisoning, `with_lock` falla con
un error pequeño y obliga a usar `recover_with`, donde la recuperación queda
nombrada en el código.

## Pruebas

Las pruebas cubren cuatro ideas:

- varios hilos pueden incrementar un contador sin perder actualizaciones;
- el lock se libera cuando termina el scope del guard;
- el poisoning es observable después de un pánico dentro de la sección crítica;
- una recuperación explícita puede reparar el valor y limpiar la señal de
  poisoning.

También hay pruebas unitarias para `try_with_lock`, porque un intento no
bloqueante debe reportar contención sin quedarse esperando.

## Benchmarks

El benchmark manual vive en [`benches/mutex_bench.rs`](../benches/mutex_bench.rs).
Compara tres cargas pequeñas:

- incremento secuencial sin mutex;
- incremento con mutex sin contención real;
- incremento con mutex compartido entre varios hilos.

Los números no buscan coronar al mutex como rápido o lento. Sirven para observar
el costo de sincronización y para recordar que una sección crítica serializa
trabajo aunque el programa tenga varios hilos.

## Ejercicios

### Ejercicio 1: Traza de contador `[Nivel 1]`

Toma un contador protegido por `EducationalMutex<i32>`. Ejecuta tres llamadas a
`with_lock` que sumen `1`, `2` y `3`. Predice el valor final y el número de
intentos bloqueantes registrados.

**Entrada/Salida esperada:** valor final `6`; intentos bloqueantes `4` si lees
el valor final con otra llamada a `with_lock`.

<details>
<summary>Pista</summary>
Leer el valor final también necesita entrar a la sección crítica.
</details>

### Ejercicio 2: Registro compartido `[Nivel 2]`

Usa `EducationalMutex<Vec<String>>` para modelar un log compartido. Lanza varios
hilos y haz que cada uno agregue una línea. Al final, comprueba que el log tiene
todas las entradas.

**Entrada/Salida esperada:** el vector final contiene una entrada por hilo.

<details>
<summary>Pista</summary>
El `Vec` debe vivir dentro de un `Arc<EducationalMutex<_>>`.
</details>

### Ejercicio 3: Recuperar poisoning `[Nivel 3]`

Provoca un pánico mientras un hilo sostiene el lock. Después usa `recover_with`
para dejar el valor protegido en un estado válido.

**Entrada/Salida esperada:** `is_poisoned()` es verdadero antes de recuperar y
falso después de recuperar.

<details>
<summary>Pista</summary>
El pánico debe ocurrir después de adquirir el guard.
</details>

### Ejercicio 4: Caché con contención `[Nivel 4]`

Diseña una caché pequeña protegida por mutex para un servicio de lectura. Decide
qué operaciones quedan dentro de la sección crítica, cuáles deben ocurrir fuera
del lock y cómo medirías si hay demasiada contención.

<details>
<summary>Pista</summary>
No hagas I/O mientras sostienes el lock. Guarda o actualiza solo el estado que
necesita exclusión.
</details>

## Soluciones

Las soluciones ejecutables de niveles 1 a 3 viven en:

- [`examples/soluciones/mutex_trace_counter.rs`](../examples/soluciones/mutex_trace_counter.rs)
- [`examples/soluciones/mutex_shared_log.rs`](../examples/soluciones/mutex_shared_log.rs)
- [`examples/soluciones/mutex_recover_poisoning.rs`](../examples/soluciones/mutex_recover_poisoning.rs)

Para el nivel 4 no hay una única solución. Una caché sana debe mantener el lock
solo para consultar o actualizar la estructura interna; el cálculo lento, la
lectura de red o el acceso a disco deben ocurrir fuera de la sección crítica.
Si la contención crece, las alternativas naturales son sharding, `RwLock`,
copy-on-write o un actor dueño de la caché.

## Referencias

- Rust Standard Library: `std::sync::Mutex`.
- Rust Book: *Fearless Concurrency*.
- Maurice Herlihy y Nir Shavit, *The Art of Multiprocessor Programming*.
- Edsger W. Dijkstra, trabajos clásicos sobre procesos cooperantes y secciones
  críticas.

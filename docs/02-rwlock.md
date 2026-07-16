# RwLock

> **Curso:** rust-concurrency · **Capítulo:** 02 · **Prerrequisitos:** `Mutex`, ownership, `Arc`, threads básicos y lectura de guards
> **Código:** [`src/rwlock.rs`](../src/rwlock.rs) · **Video:** pendiente
> **Lección en el sitio:** pendiente

## Introducción

Un `RwLock` protege un valor con dos modos de acceso: muchos lectores pueden
entrar al mismo tiempo, pero un escritor necesita exclusión total. La idea es
simple: leer no modifica la invariante, escribir sí. Cuando el sistema lee mucho
y escribe poco, separar esos dos caminos puede reducir espera innecesaria.

Este capítulo viene después de `Mutex` porque conserva la misma pregunta central
—quién puede tocar un estado compartido—, pero añade una distinción importante:
no todos los accesos tienen el mismo costo lógico. Aprenderás a reconocer cargas
de lectura dominante, exclusión de escritura, poisoning por escritores,
starvation, fairness y el peligro de intentar "subir" de lector a escritor.

## Motivación

Imagina un servicio que mantiene una tabla de configuración en memoria. Miles de
peticiones consultan si una función está habilitada; de vez en cuando, un proceso
administrativo actualiza una bandera. Con un `Mutex`, todas las lecturas se
forman en una sola fila aunque no cambien nada. El lock protege bien la
invariante, pero serializa más trabajo del necesario.

Un `RwLock` permite que varias lecturas convivan. La escritura sigue siendo
exclusiva porque modificar el mapa de configuración sí cambia la invariante. El
beneficio aparece cuando las secciones críticas de lectura son frecuentes,
cortas y no necesitan mutar el valor.

## Teoría

### Historia

Los locks de lectores/escritores existen para explotar una diferencia clásica en
sistemas concurrentes: muchas operaciones solo observan estado. Bases de datos,
cachés, archivos de configuración y estructuras administrativas suelen tener
más lecturas que escrituras. En esos casos, tratar cada lectura como si fuera una
mutación exclusiva desperdicia paralelismo.

Rust modela esta idea con `std::sync::RwLock<T>`. El guard de lectura entrega
`&T`; el guard de escritura entrega `&mut T`. La distinción no queda como
comentario ni convención: aparece en el tipo que el compilador revisa.

### Fundamentos

Un `RwLock` tiene tres estados conceptuales:

- libre;
- ocupado por uno o más lectores;
- ocupado por un escritor.

Sus invariantes principales son:

```text
si existe un escritor, no existe ningún lector ni otro escritor
si existen lectores, no existe ningún escritor
```

La lectura compartida usa `RwLockReadGuard<T>`. La escritura exclusiva usa
`RwLockWriteGuard<T>`. Ambos liberan su modo de acceso al salir de scope.

Poisoning ocurre cuando un hilo entra en pánico mientras sostiene el lock de
escritura. Un pánico durante lectura no marca el lock como poisoned, porque una
lectura no debería dejar una invariante a medias. Aun así, un lock poisoned debe
tratarse como señal de dominio: el programa necesita decidir si el valor
protegido puede seguir usándose o debe repararse.

Fairness y starvation dependen de la implementación. Algunas políticas favorecen
lectores, otras escritores, otras intentan equilibrar. El curso no promete un
orden universal de despertar hilos. La lección práctica es más humilde: si una
decisión de diseño necesita garantías fuertes de fairness, debes leer la
documentación de la primitiva concreta y medir bajo carga real.

### Casos de uso

`RwLock` es útil cuando el estado compartido tiene lecturas frecuentes y
escrituras poco frecuentes:

- catálogos o tablas de configuración en memoria;
- cachés leídas por muchos workers;
- snapshots de reglas de negocio;
- índices pequeños que se consultan mucho y se reconstruyen ocasionalmente;
- estado administrativo de servidores donde leer domina sobre actualizar.

No basta con que haya "muchos hilos". Debe haber muchas lecturas concurrentes
que puedan ejecutarse sin mutar la invariante protegida.

### Ventajas y limitaciones

Ventajas:

- Permite paralelismo entre lectores.
- Expresa en tipos la diferencia entre `&T` y `&mut T`.
- Mantiene exclusión clara para escritores.
- Encaja bien con `Arc<RwLock<T>>` en estructuras compartidas.
- Hace visibles los perfiles de carga: lectura dominante, escritura dominante o
  balanceada.

Limitaciones:

- Es más complejo que `Mutex`.
- Si hay muchas escrituras, puede comportarse como un mutex más caro.
- Puede sufrir starvation según la política de fairness y la carga.
- No debe usarse para operaciones que necesitan "leer y luego escribir" sin
  rediseñar el flujo.
- Mantener guards durante I/O o trabajo lento bloquea progreso de otros hilos.

### Comparación con alternativas

Un `Mutex` suele ser mejor cuando todas las operaciones modifican el estado o
cuando la sección crítica es tan pequeña que la complejidad extra no compensa.
Un atómico es mejor para un contador o una bandera simple. Copy-on-write puede
ser mejor cuando las lecturas requieren snapshots estables y las escrituras
pueden publicar una versión nueva. Un channel o actor puede ser mejor cuando un
solo dueño del estado simplifica el diseño.

`RwLock` no es "mutex rápido". Es una primitiva con una apuesta concreta:
permitir lectores concurrentes a cambio de una política de coordinación más
sofisticada.

## Diagramas

El diagrama principal vive en [`diagrams/02-rwlock.mmd`](../diagrams/02-rwlock.mmd).
Muestra lectores que pueden compartir el lock y un escritor que solo entra
cuando no queda ningún guard activo.

## Análisis de complejidad

| Operación | Mejor caso | Caso promedio | Peor caso | Espacio |
|-----------|------------|---------------|-----------|---------|
| `new` | O(1) | O(1) | O(1) | O(1) adicional |
| `read` | O(1) sin contención | depende del scheduler y escritores pendientes | espera no acotada si no hay progreso | O(1) |
| `write` | O(1) sin contención | depende de lectores/escritores activos | espera no acotada si no hay progreso | O(1) |
| `with_read` | O(1) + costo del closure | depende del scheduler y del closure | espera no acotada si no hay progreso | O(1) |
| `with_write` | O(1) + costo del closure | depende del scheduler y del closure | espera no acotada si no hay progreso | O(1) |
| `try_with_read` | O(1) | O(1) | O(1) | O(1) |
| `try_with_write` | O(1) | O(1) | O(1) | O(1) |
| `recover_write_with` | O(1) + costo del closure | depende del scheduler y del closure | espera no acotada si no hay progreso | O(1) |
| `observations` | O(1) | O(1) | O(1) | O(1) |

La complejidad algorítmica no cuenta toda la historia. El costo real depende de
la mezcla de lecturas y escrituras, de cuánto duran los guards y de la política
del sistema para despertar hilos.

## Visualización interactiva (opcional)

No aplica todavía. Una visualización futura podría mostrar lectores agrupados,
un escritor esperando y diferentes políticas de fairness. Por ahora, las pruebas
y el diagrama cubren el modelo mental necesario.

## Implementación

La implementación del curso usa `EducationalRwLock<T>`, un wrapper pedagógico
sobre `std::sync::RwLock<T>`. No implementa una primitiva desde cero. Su valor
está en nombrar las operaciones que queremos discutir:

- `read` y `with_read` para acceso compartido;
- `write` y `with_write` para acceso exclusivo;
- `try_with_read` y `try_with_write` para observar contención sin bloquear;
- `recover_write_with` para reparar poisoning con intención explícita;
- `observations` para contar intentos y señales de contención.

La recuperación exige acceso de escritura porque reparar una invariante suele
necesitar mutar el valor protegido. Esa decisión mantiene el capítulo alineado
con la idea central: leer observa, escribir cambia.

## Pruebas

Las pruebas cubren:

- varios lectores observando el mismo snapshot;
- un escritor que no entra mientras existe un lector activo;
- visibilidad de escrituras para lectores posteriores;
- poisoning provocado por un pánico con guard de escritura;
- recuperación explícita y limpieza del estado poisoned;
- corrección de una carga concurrente con muchas lecturas y algunas escrituras.

También hay pruebas unitarias para los helpers de lectura y para contención de
escritura detectada con `try_with_write`.

## Benchmarks

El benchmark manual vive en [`benches/rwlock_bench.rs`](../benches/rwlock_bench.rs).
Mide tres perfiles pequeños:

- carga de lectura dominante;
- carga de escritura dominante;
- carga balanceada con lecturas y escrituras mezcladas.

El resultado no debe leerse como una ley universal. Sirve para formular una
pregunta de ingeniería: si tu carga no es realmente de lectura dominante, quizá
`RwLock` solo añadió complejidad.

## Ejercicios

### Ejercicio 1: Traza de lectura `[Nivel 1]`

Crea un `EducationalRwLock<Vec<i32>>`, haz dos lecturas con `with_read` y predice
el número de intentos de lectura registrados.

**Entrada/Salida esperada:** longitud `3`, suma `6`, intentos de lectura `2`.

<details>
<summary>Pista</summary>
Cada llamada a `with_read` adquiere un guard de lectura.
</details>

### Ejercicio 2: Catálogo compartido `[Nivel 2]`

Modela un catálogo de capítulos con `Arc<EducationalRwLock<Vec<String>>>`.
Agrega un capítulo con escritura exclusiva y luego lanza varios lectores que
impriman el snapshot.

**Entrada/Salida esperada:** todos los lectores observan el catálogo actualizado.

<details>
<summary>Pista</summary>
Haz la escritura antes de lanzar lectores si quieres un resultado determinista.
</details>

### Ejercicio 3: Recuperar poisoning `[Nivel 3]`

Provoca un pánico mientras un hilo sostiene el lock de escritura. Después usa
`recover_write_with` para dejar el valor en un estado válido.

**Entrada/Salida esperada:** `is_poisoned()` es verdadero antes de recuperar y
falso después de recuperar.

<details>
<summary>Pista</summary>
Un pánico con guard de lectura no marca el lock como poisoned; necesitas un
guard de escritura.
</details>

### Ejercicio 4: Evitar upgrade implícito `[Nivel 4]`

Diseña una operación que necesita revisar un valor y quizá escribir. Explica por
qué sostener un guard de lectura mientras intentas adquirir escritura puede
bloquear el sistema, y propone una alternativa.

<details>
<summary>Pista</summary>
Lee un snapshot pequeño, libera el guard, decide, y luego intenta escribir
validando de nuevo la condición.
</details>

## Soluciones

Las soluciones ejecutables de niveles 1 a 3 viven en:

- [`examples/soluciones/rwlock_read_trace.rs`](../examples/soluciones/rwlock_read_trace.rs)
- [`examples/soluciones/rwlock_shared_catalog.rs`](../examples/soluciones/rwlock_shared_catalog.rs)
- [`examples/soluciones/rwlock_recover_poisoning.rs`](../examples/soluciones/rwlock_recover_poisoning.rs)

Para el nivel 4 no hay una única respuesta. Una alternativa sana es separar la
operación en dos fases: leer lo necesario, soltar el guard, decidir si hace falta
escribir, adquirir el guard de escritura y validar de nuevo antes de mutar. Si
esa doble validación complica demasiado el dominio, quizá el estado necesita un
actor, un channel o una API que exprese la operación completa como escritura.

## Referencias

- Rust Standard Library: `std::sync::RwLock`.
- Rust Book: *Fearless Concurrency*.
- Maurice Herlihy y Nir Shavit, *The Art of Multiprocessor Programming*.
- Bryan Cantrill, discusiones públicas sobre concurrencia, locks y observación
  de sistemas.

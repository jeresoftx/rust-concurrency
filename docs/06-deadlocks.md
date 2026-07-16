# Deadlocks

> **Curso:** rust-concurrency · **Capítulo:** 06 · **Prerrequisitos:** `Mutex`, `RwLock`, `Arc`, grafos básicos y razonamiento sobre secciones críticas
> **Código:** [`src/deadlocks.rs`](../src/deadlocks.rs) · **Video:** pendiente
> **Lección en el sitio:** pendiente

## Introducción

Un deadlock ocurre cuando un conjunto de hilos queda esperando para siempre
porque cada uno necesita un recurso que otro hilo del mismo conjunto no libera.
El programa no necesariamente consume CPU ni falla con un panic: simplemente
deja de avanzar.

Este capítulo enseña las condiciones de Coffman, grafos wait-for, orden total de
locks, estrategias con `try_lock` o tiempos límite, y diseño de APIs que hace
difícil crear esperas circulares. La meta no es memorizar recetas. La meta es
aprender a ver la dependencia invisible detrás de cada lock.

## Motivación

Imagina una transferencia entre cuentas. Un hilo transfiere de A hacia B y
adquiere primero el lock de A. Al mismo tiempo, otro hilo transfiere de B hacia
A y adquiere primero el lock de B. Si ambos intentan adquirir el segundo lock
sin soltar el primero, cada uno espera al otro.

La solución sana no empieza con un tiempo límite mágico. Empieza preguntando
cuál es la invariante: una transferencia necesita modificar dos saldos como una
operación atómica a nivel de negocio. Si todas las transferencias adquieren
locks en el mismo orden global, dos hilos pueden esperar, pero no pueden formar
un ciclo.

## Teoría

### Historia

Los deadlocks aparecen desde los primeros sistemas operativos con recursos
compartidos: memoria, dispositivos, archivos, semáforos y locks. El problema no
pertenece a Rust; pertenece a cualquier sistema donde varias actividades
compiten por recursos exclusivos.

Edward G. Coffman formuló cuatro condiciones necesarias para que exista un
deadlock. Son útiles porque convierten una falla difusa en una lista concreta:
si rompes al menos una condición, evitas el deadlock.

### Fundamentos

Las condiciones de Coffman son:

- exclusión mutua: al menos un recurso no puede compartirse;
- hold and wait: un hilo sostiene un recurso mientras espera otro;
- no preemption: el recurso no puede quitarse a la fuerza;
- espera circular: existe un ciclo de hilos donde cada uno espera a otro.

En Rust seguro, `Mutex` y `RwLock` ayudan con data races, pero no eliminan
deadlocks lógicos. El compilador no sabe que `account_a` debe adquirirse antes
que `account_b`, ni puede deducir todos los órdenes posibles de locks en un
sistema grande.

Un grafo wait-for representa dependencias de espera. Un borde `A -> B` significa
que A espera a B. Si el grafo contiene un ciclo, existe una espera circular y,
si las otras condiciones se sostienen, hay un deadlock posible.

La prevención por orden total asigna un rango a cada recurso. El protocolo dice:
un hilo solo puede adquirir locks con rango igual o mayor al mayor rango que ya
sostiene. Así se rompe la espera circular porque las aristas siempre avanzan en
una dirección.

La invariante central del capítulo es:

```text
si todos los hilos adquieren recursos siguiendo el mismo orden total, no puede
existir un ciclo de espera entre esos recursos
```

### Casos de uso

El razonamiento sobre deadlocks importa en:

- transferencias financieras o movimientos entre dos entidades;
- cachés con varios mapas internos;
- índices de bases de datos protegidos por múltiples locks;
- servicios que combinan locks de sesión, usuario y recurso;
- pipelines con backpressure y recursos limitados;
- código de infraestructura donde una espera silenciosa es peor que un error.

### Ventajas y limitaciones

Ventajas:

- El orden total es simple de auditar.
- Los grafos wait-for ayudan a explicar y detectar ciclos.
- `try_lock` y los tiempos límite pueden convertir una espera infinita en una
  decisión.
- Diseñar APIs de alto nivel oculta combinaciones peligrosas de locks.

Limitaciones:

- Un orden total rígido puede ser incómodo si el dominio cambia.
- `try_lock` mal usado puede crear livelock o reintentos costosos.
- Los tiempos límite no corrigen la causa; solo limitan el daño.
- Detectar ciclos en producción requiere instrumentación y decisiones de
  recuperación.
- Las pruebas no exploran todos los interleavings posibles.

### Comparación con alternativas

Prevención significa diseñar para que el deadlock no pueda formarse: orden total,
un solo lock por invariante o APIs que encapsulan la adquisición múltiple.

Evitación significa decidir en tiempo de ejecución si una adquisición es segura.
Es más flexible, pero necesita más estado y suele ser difícil de justificar en
servicios normales.

Detección significa permitir esperas y buscar ciclos. Es útil en bases de datos,
donde el sistema puede abortar una transacción. En una aplicación común, detectar
sin una política clara de recuperación solo confirma que el programa está
atorado.

Recuperación significa romper el ciclo: abortar una operación, liberar recursos,
reiniciar un worker o degradar un flujo. Es una decisión de producto y de
arquitectura, no solo de código.

Los canales y los diseños tipo actor reducen la necesidad de compartir locks, pero no
eliminan todos los bloqueos posibles: también puede haber ciclos de mensajes,
backpressure mal diseñada o esperas de respuesta cruzadas.

## Diagramas

El diagrama principal vive en
[`diagrams/06-deadlocks.mmd`](../diagrams/06-deadlocks.mmd). Muestra dos hilos
que esperan recursos cruzados, el grafo wait-for resultante y la prevención por
orden total de locks.

## Análisis de complejidad

| Operación | Mejor caso | Caso promedio | Peor caso | Espacio |
|-----------|------------|---------------|-----------|---------|
| `LockRank::new` | O(1) | O(1) | O(1) | O(1) |
| `LockOrderTracker::new` | O(1) | O(1) | O(1) | O(1) |
| `LockOrderTracker::enter` | O(1) | O(k) por buscar máximo en locks sostenidos | O(k) | O(1) adicional |
| `LockOrderTracker::exit` | O(1) | O(k) | O(k) | O(1) adicional |
| `LockOrderTracker::held_ranks` | O(k) | O(k) | O(k) | O(k) |
| `WaitForGraph::add_wait` | O(log V + log E) | O(log V + log E) | O(log V + log E) | O(V + E) acumulado |
| `WaitForGraph::has_cycle` | O(V + E) | O(V + E) | O(V + E) | O(V) |
| `WaitForGraph::cycle_path` | O(V + E) | O(V + E) | O(V + E) | O(V) |
| `BankAccounts::new` | O(n) | O(n) | O(n) | O(n) |
| `BankAccounts::transfer_ordered` | O(1) para dos cuentas | O(1) más espera por locks | O(1) más contención no acotada | O(1) |
| `BankAccounts::balance` | O(1) | O(1) más espera por lock | O(1) más contención no acotada | O(1) |

`transfer_ordered` no promete wait-freedom ni lock-freedom. Promete que la
adquisición de dos cuentas sigue un orden total por índice, lo cual elimina la
espera circular entre esas cuentas.

## Visualización interactiva (opcional)

No aplica todavía. Una visualización futura debería permitir arrastrar recursos,
crear bordes de espera y observar cuándo aparece un ciclo en el grafo wait-for.

## Implementación

La implementación del curso define:

- `LockRank`: rango educativo para ordenar recursos;
- `LockOrderTracker`: validador de protocolo para adquisición en orden;
- `LockOrderViolation`: error cuando un hilo intenta invertir el orden;
- `WaitForGraph`: grafo determinista para detectar ciclos de espera;
- `BankAccounts`: ejemplo de transferencia con locks adquiridos por índice;
- `TransferError`: errores de dominio para la transferencia educativa.

La API evita `unsafe`. El ejemplo de cuentas no intenta ser un banco real:
existe para aislar una invariante, mostrar la adquisición ordenada de dos locks
y comprobar que el orden no depende de la dirección de la transferencia.

## Pruebas

Las pruebas cubren:

- aceptación de adquisición con rangos crecientes;
- rechazo de inversión de orden;
- liberación y reentrada;
- detección de ciclos en grafos wait-for;
- grafos acíclicos;
- transferencia ordenada entre cuentas;
- error por fondos insuficientes.

## Benchmarks

El benchmark manual vive en
[`benches/deadlocks_bench.rs`](../benches/deadlocks_bench.rs). Mide dos costos:
validar orden de locks con `LockOrderTracker` y ejecutar transferencias
ordenadas entre dos cuentas.

La medición no demuestra ausencia de deadlocks. Solo ayuda a discutir el costo
pedagógico de una estrategia preventiva. La correctitud vive en la invariante de
orden total y en las pruebas que la hacen observable.

## Ejercicios

### Ejercicio 1: Orden válido `[Nivel 1]`

Crea dos `LockRank`, adquiere primero el menor y luego el mayor con
`LockOrderTracker`.

**Entrada/Salida esperada:** ambos `enter` devuelven `Ok(())` y el rastreador
contiene dos locks.

<details>
<summary>Pista</summary>
El rango numérico debe crecer o mantenerse igual.
</details>

### Ejercicio 2: Detectar ciclo `[Nivel 2]`

Construye un `WaitForGraph` con los bordes `a -> b`, `b -> c` y `c -> a`.

**Entrada/Salida esperada:** `has_cycle()` devuelve `true` y `cycle_path()`
devuelve un ciclo.

<details>
<summary>Pista</summary>
Un ciclo puede incluir el nodo inicial repetido al final para cerrar el camino.
</details>

### Ejercicio 3: Transferencia ordenada `[Nivel 3]`

Crea dos cuentas y transfiere saldo de la cuenta 0 a la cuenta 1 usando
`transfer_ordered`.

**Entrada/Salida esperada:** los saldos cambian y `last_lock_order()` muestra
los índices en orden ascendente.

<details>
<summary>Pista</summary>
El orden de adquisición no depende de cuál cuenta sea origen.
</details>

### Ejercicio 4: Política de recuperación `[Nivel 4]`

Diseña una política para un servicio que detecta ciclos en un grafo wait-for.
Decide si debe abortar una operación, reintentar con backoff, registrar un
incidente o reiniciar un worker. Justifica el costo para el usuario y para la
integridad del sistema.

<details>
<summary>Pista</summary>
La detección solo encuentra el ciclo; la recuperación define qué daño aceptas
para romperlo.
</details>

## Soluciones

Las soluciones ejecutables de niveles 1 a 3 viven en:

- [`examples/soluciones/deadlocks_lock_order.rs`](../examples/soluciones/deadlocks_lock_order.rs)
- [`examples/soluciones/deadlocks_wait_for_cycle.rs`](../examples/soluciones/deadlocks_wait_for_cycle.rs)
- [`examples/soluciones/deadlocks_ordered_transfer.rs`](../examples/soluciones/deadlocks_ordered_transfer.rs)

Para el nivel 4, una respuesta sana nombra explícitamente qué operación puede
perder, cuál puede reintentarse y qué evidencia queda para depurar. En bases de
datos suele ser aceptable abortar una transacción. En un servicio de usuario,
quizá conviene rechazar una operación con error claro antes que mantener una
espera indefinida.

## Referencias

- Rust Standard Library: `std::sync::Mutex` y `std::sync::RwLock`.
- Edward G. Coffman Jr., Melanie Elphick y Arie Shoshani, *System Deadlocks*.
- Libros de sistemas operativos: prevención, evitación, detección y recuperación
  de deadlocks.
- Mara Bos, *Rust Atomics and Locks*.

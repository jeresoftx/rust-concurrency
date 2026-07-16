# Epoch GC

> **Curso:** rust-concurrency · **Capítulo:** 10 · **Prerrequisitos:**
> estructuras lock-free, ciclos CAS, ABA, ownership en Rust, `Arc`, `Mutex`,
> memory ordering básico y hazard pointers
> **Código:** [`src/epoch_gc.rs`](../src/epoch_gc.rs) · **Video:** pendiente
> **Lección en el sitio:** pendiente

## Introducción

Epoch GC, o recolección por épocas, resuelve el mismo problema que hazard
pointers desde otro ángulo: cuándo puede liberarse memoria retirada por una
estructura concurrente. En vez de proteger nodos individuales, cada participante
anuncia la época global en la que entró a una sección crítica. Los objetos
retirados esperan hasta que las épocas avanzan y ningún participante antiguo
puede seguir observándolos.

Este capítulo enseña épocas fijadas, avance global, objetos retirados, estados
quiescentes, participantes estancados y retención de memoria. El modelo usa
identificadores seguros para enseñar las invariantes antes de estudiar versiones
reales con punteros y `unsafe`.

## Motivación

Una pila lock-free puede retirar un nodo con un CAS exitoso, pero no puede
liberarlo de inmediato si otro hilo pudo leerlo antes del cambio. Hazard
pointers piden publicar cada nodo protegido. Epoch GC elige una unidad más
grande: una sección crítica completa.

El contrato cambia a:

```text
entro a una sección crítica y fijo la época actual
leo estructuras compartidas
salgo y aviso que estoy quiescente
los escritores retiran objetos con la época actual
el dominio avanza cuando no hay participantes antiguos
solo reclama objetos suficientemente viejos y no observables
```

La ventaja es que leer puede ser barato: fijar una época cubre muchas lecturas.
El costo aparece cuando un participante se queda fijado demasiado tiempo. Ese
participante puede impedir avance y retener memoria retirada aunque la
estructura pública ya no la use.

## Teoría

### Historia

La recolección por épocas aparece en estructuras lock-free, motores de base de
datos, colas concurrentes y bibliotecas de concurrencia. Su idea es pragmática:
si todos los participantes que pudieron ver una versión vieja ya salieron de su
sección crítica, entonces los objetos retirados antes de esa salida pueden
liberarse.

Rust no tiene un recolector de basura general. Cuando una estructura de bajo
nivel usa punteros compartidos, debe demostrar que no habrá lectura después de
liberar. Epoch GC aporta un protocolo para esa demostración.

### Fundamentos

Una época global es un contador lógico. Un participante fijado registra:

```text
participante -> época observada al entrar
```

Un objeto retirado registra:

```text
objeto -> época en que salió de la estructura pública
```

El dominio puede avanzar la época si no hay participantes fijados en épocas
anteriores a la global. Un participante fijado en la época actual puede permitir
un primer avance, porque todavía no es antiguo. Si permanece fijado después del
avance, queda estancado y bloquea el siguiente avance.

La reclamación requiere dos condiciones:

1. el objeto retirado cumplió el rezago configurado;
2. ningún participante sigue fijado en la época del retiro o en una anterior.

La invariante central del capítulo es:

```text
un objeto retirado solo puede reclamarse cuando todas las secciones críticas
que pudieron observarlo ya llegaron a estado quiescente
```

### Casos de uso

Epoch GC aparece en:

- pilas, colas y listas lock-free;
- mapas concurrentes con muchas lecturas cortas;
- motores de almacenamiento con versiones retiradas;
- sistemas donde el costo por lectura debe ser bajo;
- estructuras que agrupan muchas lecturas dentro de una sección crítica;
- bibliotecas que pueden exigir disciplina de entrada y salida por hilo.

### Ventajas y limitaciones

Ventajas:

- Una marca de época puede cubrir muchas lecturas.
- El costo por acceso puede ser menor que publicar hazard pointers por nodo.
- El avance por épocas agrupa la reclamación.
- Es natural para operaciones cortas y frecuentes.

Limitaciones:

- Un participante estancado puede retener mucha memoria.
- El protocolo depende de que todos hagan `unpin`.
- Es menos preciso que proteger nodos individuales.
- No evita por sí solo errores de ordenamiento de memoria.
- En implementaciones reales suele requerir `unsafe` y auditoría cuidadosa.

### Comparación con alternativas

Hazard pointers protegen nodos concretos. Dan más precisión: si nadie protege un
nodo, puede reclamarse aunque otros participantes sigan activos. A cambio, cada
lectura protegida paga publicación y validación.

El conteo de referencias es directo para propiedad compartida, pero cada clon y
drop puede costar operaciones atómicas. También modela propiedad, no solo
observación temporal, y puede introducir ciclos si se diseña mal.

La reclamación con candados puede ser más simple: si un candado exclusivo cubre
lectura, retiro y liberación, no hay lector concurrente. El costo es perder
progreso lock-free y aceptar contención o interbloqueos si el diseño se complica.

Un recolector de basura por trazado simplifica la vida del programador, pero
cambia el entorno de ejecución y no existe como mecanismo general de Rust.

## Diagramas

El diagrama principal vive en
[`diagrams/10-epoch-gc.mmd`](../diagrams/10-epoch-gc.mmd). Muestra entrada a
sección crítica, retiro de objetos, avance global, participante estancado y
reclamación eventual.

## Análisis de complejidad

| Operación | Mejor caso | Caso promedio | Peor caso | Espacio |
|-----------|------------|---------------|-----------|---------|
| `ParticipantId::new` | O(1) | O(1) | O(1) | O(1) |
| `ObjectId::new` | O(1) | O(1) | O(1) | O(1) |
| `RetiredObject::new` | O(1) | O(tamaño de carga) | O(tamaño de carga) | O(tamaño de carga) |
| `EpochDomain::new` | O(1) | O(1) | O(1) | O(1) |
| `EpochDomain::global_epoch` | O(1) | O(1) | O(1) | O(1) |
| `EpochDomain::pin` | O(log p) | O(log p) | O(log p) | O(1) adicional |
| `EpochDomain::unpin` | O(log p) | O(log p) | O(log p) | O(1) |
| `EpochDomain::is_pinned` | O(log p) | O(log p) | O(log p) | O(1) |
| `EpochDomain::pinned_participants` | O(p) | O(p) | O(p) | O(p) |
| `EpochDomain::retire` | O(1) | O(1) | O(1) | O(1) más carga |
| `EpochDomain::try_advance` | O(p) | O(p) | O(p) | O(p) |
| `EpochDomain::scan` | O(r * p) | O(r * p) | O(r * p) | O(r + p) |
| `EpochDomain::retired_objects` | O(r) | O(r) | O(r) | O(r) |
| `EpochDomain::reclaimed_objects` | O(n) | O(n) | O(n) | O(n) |

`p` es el número de participantes, `r` el número de objetos retirados y `n` el
historial de objetos reclamados.

## Visualización interactiva (opcional)

No aplica todavía. Una visualización futura debería permitir fijar
participantes, retirar objetos por época, avanzar la época global y observar
cuánta memoria queda retenida cuando alguien no llega a estado quiescente.

## Implementación

La implementación del curso define:

- `ParticipantId`: participante lógico de una sección crítica;
- `ObjectId`: identificador seguro de objeto retirado;
- `Pin`: marca de participante fijado en una época;
- `RetiredObject`: objeto retirado con época y carga descriptiva;
- `EpochAdvance`: resultado de intentar avanzar la época global;
- `ReclaimReport`: resultado de escanear objetos retirados;
- `EpochDomain`: dominio con época global, participantes fijados, retirados y
  reclamados.

La API evita `unsafe`. En una implementación real, `ObjectId` podría representar
memoria dinámica retirada por una estructura lock-free. Aquí es un identificador
para enseñar el protocolo sin exponer al estudiante a lectura de memoria
liberada.

## Pruebas

Las pruebas cubren:

- fijar participantes y observar su época;
- marcar quiescencia con `unpin`;
- avance global permitido y avance bloqueado;
- retiro de objetos;
- retraso por rezago de épocas;
- bloqueo por participante estancado;
- reclamación eventual;
- reclamación ordenada de objetos retirados en épocas distintas.

## Benchmarks

El benchmark manual vive en
[`benches/epoch_gc_bench.rs`](../benches/epoch_gc_bench.rs). Mide costo de
`pin`/`unpin`, rendimiento de retiro y comportamiento de reclamación retrasada
por participantes estancados.

El objetivo es mirar el protocolo, no declarar una estrategia universalmente
mejor. Epoch GC suele brillar cuando las secciones críticas son cortas y
frecuentes; si un participante se queda fijado, la memoria retenida puede crecer
sin que la estructura pública crezca.

## Ejercicios

### Ejercicio 1: Fijar y liberar una época `[Nivel 1]`

Crea un dominio, fija un participante y después márcalo quiescente.

**Entrada/Salida esperada:** `is_pinned(participant)` pasa de `true` a `false`.

<details>
<summary>Pista</summary>
`pin` registra al participante; `unpin` elimina ese registro.
</details>

### Ejercicio 2: Reclamación retrasada `[Nivel 2]`

Retira un objeto en época 0 con rezago 2. Escanea antes y después de dos avances.

**Entrada/Salida esperada:** primero aparece en `delayed`; después aparece en
`reclaimed`.

<details>
<summary>Pista</summary>
El rezago se mide contra la época global, no contra el número de escaneos.
</details>

### Ejercicio 3: Participante estancado `[Nivel 3]`

Fija un participante, retira un objeto y avanza una vez. Intenta avanzar de
nuevo y escanea.

**Entrada/Salida esperada:** el segundo avance se bloquea y el escaneo reporta
al participante en `blocked_by`.

<details>
<summary>Pista</summary>
Un participante fijado en una época anterior a la global todavía podría observar
objetos retirados en esa época.
</details>

### Ejercicio 4: Elegir estrategia de reclamación `[Nivel 4]`

Compara epoch GC, hazard pointers, conteo de referencias y reclamación con
candados para una cola lock-free usada por 64 hilos con lectores muy cortos y
un lector ocasional que puede tardar segundos.

<details>
<summary>Pista</summary>
Incluye costo por lectura, memoria retenida, latencia de reclamación y facilidad
de auditar el protocolo.
</details>

## Soluciones

Las soluciones ejecutables de niveles 1 a 3 viven en:

- [`examples/soluciones/epoch_pin_unpin.rs`](../examples/soluciones/epoch_pin_unpin.rs)
- [`examples/soluciones/epoch_delayed_reclaim.rs`](../examples/soluciones/epoch_delayed_reclaim.rs)
- [`examples/soluciones/epoch_stalled_participant.rs`](../examples/soluciones/epoch_stalled_participant.rs)

Para el nivel 4, una respuesta sana no elige por moda. Si las lecturas son
cortas y disciplinadas, epoch GC puede reducir costo por acceso. Si hay lectores
largos o impredecibles, hazard pointers pueden limitar mejor la memoria retenida
porque protegen nodos concretos. El conteo de referencias simplifica propiedad
compartida, pero paga operaciones atómicas por clon/drop. La reclamación con
candados reduce riesgo conceptual, pero cambia las garantías de progreso.

## Referencias

- Keir Fraser, *Practical Lock-Freedom*.
- Maged M. Michael, *Hazard Pointers: Safe Memory Reclamation for Lock-Free Objects*.
- Maurice Herlihy y Nir Shavit, *The Art of Multiprocessor Programming*.
- Mara Bos, *Rust Atomics and Locks*.
- Rust Standard Library: `std::sync::atomic`.

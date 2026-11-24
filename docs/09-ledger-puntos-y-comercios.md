# 09 - Ledger, Puntos y Comercios

Corresponde a la Fase 4. Requiere las compuertas G2 (humanidad) y G3 (nodo) aprobadas, un **dictamen legal** y una **auditoría externa** antes de que un solo punto tenga valor.

## 1. Principios

1. **Cada punto es una obligación.** Se emite con presupuesto, no por fórmula abierta.
2. **Enteros siempre.** La unidad interna es el milipunto (1 punto = 1000 milipuntos). Ningún número decimal toca el ledger.
3. **Solo inserciones.** No se actualiza ni se borra un asiento. Un error se corrige con un asiento inverso.
4. **Doble entrada.** Cada asiento suma cero.
5. **Aislado.** Servicio propio, base de datos propia, credenciales propias. Ningún otro servicio escribe en ella.
6. **Auditable.** Cadena de hashes y raíz diaria publicada.

## 2. Reglas de producto

| Regla | Valor inicial | Motivo |
| :-- | :-- | :-- |
| Transferencia entre usuarios | Prohibida | Evita mercado negro y reduce el riesgo regulatorio de servicio de pagos. |
| Vencimiento | 12 meses desde la emisión | Acota la obligación acumulada. |
| Tope de saldo | Configurable | Limita la pérdida por robo de llave. |
| Canje | Solo en comercios aliados | |
| Compra de puntos | Prohibida | Evita que sean dinero electrónico. |
| Una billetera por identidad de nivel 3 | Sí | El tope de emisión se aplica por humano único. Los niveles 1 y 2 no tienen billetera (D-15). |

## 3. Emisión

Cada día se reparte un **presupuesto diario** `B` fijado por el proyecto.

```
crédito_i   = min(GB·hora auditados del humano i, tope_por_humano)
puntos_i    = floor(B × crédito_i / Σ crédito)
resto       → cuenta de reserva del sistema
```

- Si participan pocos nodos, cada uno gana más; si participan muchos, menos. La obligación diaria total es siempre `B`.
- `tope_por_humano` impide que un operador grande se lleve todo. Valor inicial: el equivalente a 50 GB durante 24 horas.
- Si el crédito total del día es menor que un mínimo, no se emite: evita que dos nodos se repartan el presupuesto entero.
- En F3 se usa el mismo cálculo con **puntos de prueba** sin valor, para calibrar `B` y los topes con datos reales.

## 4. Modelo de datos

```sql
CREATE TABLE ledger_accounts (
  id       uuid PRIMARY KEY,
  kind     text NOT NULL,     -- wallet | merchant | sys_emission | sys_reserve | sys_expired
  owner    bytea,             -- llave pública de billetera o id de comercio
  created  timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE journal (
  seq        bigserial PRIMARY KEY,
  id         uuid NOT NULL UNIQUE,
  kind       text NOT NULL,   -- emission | redeem | expire | reversal
  idem_key   text NOT NULL UNIQUE,
  created    timestamptz NOT NULL DEFAULT now(),
  meta       jsonb NOT NULL,
  prev_hash  bytea NOT NULL,
  hash       bytea NOT NULL   -- sha256(prev_hash ‖ contenido canónico del asiento)
);

CREATE TABLE postings (
  journal_id uuid NOT NULL REFERENCES journal(id),
  account_id uuid NOT NULL REFERENCES ledger_accounts(id),
  amount     bigint NOT NULL, -- milipuntos, con signo
  PRIMARY KEY (journal_id, account_id)
);

CREATE TABLE balances (
  account_id uuid PRIMARY KEY REFERENCES ledger_accounts(id),
  amount     bigint NOT NULL
);
```

**Invariantes (cada una con su prueba automática)**

1. Para cada asiento, la suma de sus apuntes es cero.
2. La suma de todos los saldos es cero.
3. El saldo de una cuenta `wallet` o `merchant` nunca es negativo.
4. `balances` es igual a la suma de `postings` por cuenta. Se comprueba en una conciliación periódica.
5. La cadena de hashes no tiene huecos.
6. Los permisos de base de datos impiden `UPDATE` y `DELETE` sobre `journal` y `postings`.
7. Solo la función de asiento modifica `balances`, en la misma transacción que inserta los apuntes.

**Concurrencia:** toda operación corre con aislamiento serializable y reintento ante conflicto. La clave de idempotencia hace que repetir una petición devuelva el resultado original.

## 5. Canje

El comercio presenta el QR. El usuario aprueba.

```
Comercio (TPV)                 Ledger                    Billetera (usuario)
     │  1. crear cobro             │                              │
     ├────────────────────────────▶│                              │
     │  2. cobro {id, importe,     │                              │
     │     comercio, expira}       │                              │
     │◀────────────────────────────┤                              │
     │  3. muestra QR  ─────────────────────────────────────────▶ │ escanea
     │                             │  4. consulta el cobro        │
     │                             │◀─────────────────────────────┤
     │                             │  5. intención firmada        │
     │                             │◀─────────────────────────────┤
     │                             │  6. valida y asienta         │
     │  7. confirmación            │  7. confirmación             │
     │◀────────────────────────────┤─────────────────────────────▶│
```

- El cobro es de un solo uso y caduca a los 120 segundos.
- La billetera muestra nombre del comercio e importe **obtenidos del ledger**, no del QR, antes de firmar.
- La intención firmada contiene: identificador del cobro, importe, comercio, caducidad y un valor único.
- El paso 6 es una única transacción: valida firma, vigencia, saldo y unicidad, y asienta el débito y el crédito.
- Sin conexión no hay canje en la versión 1.
- Las peticiones de la billetera van por la ruta que oculta la IP. El comercio ve un identificador de operación, nunca la billetera.

## 6. Antifraude

| Riesgo | Control |
| :-- | :-- |
| Repetición del QR o de la intención | Un solo uso, caducidad corta, clave de idempotencia. |
| Doble gasto simultáneo | Aislamiento serializable y restricción de saldo. |
| Colusión entre nodo y comercio para "lavar" puntos | Topes por humano y por día; liquidación del comercio con retraso; detección de pares billetera-comercio repetidos. |
| Comercio fantasma | Verificación de negocio manual antes del alta; límite inicial bajo. |
| Robo de llave de billetera | Tope de saldo; tope diario de canje; confirmación en el dispositivo. |
| Error o abuso interno | Solo inserciones; cadena de hashes; raíz publicada; doble aprobación para asientos manuales. |

## 7. Comercios

- **Alta:** verificación manual del negocio, aceptación de condiciones, límites iniciales.
- **Panel:** cobros, historial, conciliación, exportación.
- **TPV:** aplicación web instalable; no exige hardware especial.
- **Roles:** dueño, cajero (solo cobra), contable (solo lee).

**Modelos económicos** (decisión ejecutiva `15` D-03):

| Modelo | Cómo funciona | Riesgo |
| :-- | :-- | :-- |
| **A. Descuento financiado por el comercio** (por defecto) | El comercio acepta puntos como descuento a cambio de visibilidad local y herramientas. No hay pago en dinero del proyecto al comercio. | Bajo. Menos atractivo para el comercio. |
| **B. Liquidación en dinero** | El proyecto paga al comercio por los puntos canjeados y cobra una comisión. | Alto: exige fondos, y puede requerir licencia. Solo con dictamen legal favorable. |

El ledger es el mismo en ambos modelos; cambia la liquidación.

**Piloto en Colombia.** Los importes en dinero del comercio (valor de la compra, descuento aplicado) se guardan en **pesos colombianos como enteros, sin decimales**, y nunca se mezclan con milipuntos en una misma columna. Las condiciones del descuento se muestran completas antes de canjear, como una promoción al consumidor. El régimen de los puntos, su tratamiento fiscal y las obligaciones del comercio los fija el dictamen de F4-T00 (`10` §6).

## 8. Transparencia

- Cada día se publica la raíz de la cadena de hashes y los totales: emitido, canjeado, vencido y en circulación.
- Cualquier billetera puede pedir la prueba de que sus asientos están en la cadena.
- Ningún dato publicado permite identificar una billetera.

## 9. Evolución hacia blockchain

No es parte del plan comprometido. Si llega a evaluarse en F5:

1. Dictamen legal previo en cada jurisdicción donde se opere.
2. ADR con el modelo de amenazas del puente.
3. El ledger interno sigue siendo la fuente de verdad hasta completar una migración auditada.

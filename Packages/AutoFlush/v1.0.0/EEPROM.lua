-- Auto-empty Industrial Fluid Buffer(s) when >= 50% full.
-- Now with logging on each poll to confirm readings.

-- === Configuration ===
local TANK_NICK_FILTER     = "AutoFlushIFB"  -- nick you set on your tank(s)
local CAPACITY_M3          = 2400            -- Fluid Buffer capacity (m^3)
local THRESHOLD_PCT        = 0.50            -- trigger threshold (50%)
local POLL_SECONDS         = 10              -- how often to check (seconds)
local POST_FLUSH_COOLDOWN  = 30              -- avoid rapid re-flush loops (seconds)
local DEBUG_CONNECTORS     = false           -- set true to print per-connector details

-- === Locate tanks ===
local ids = component.findComponent(TANK_NICK_FILTER)
if not ids or #ids == 0 then
  error("No components found for nick '" .. TANK_NICK_FILTER .. "'.")
end
local tanks = component.proxy(ids)

-- === Helpers ===

-- Read all pipe connectors of a tank and return:
--   maxContent (m^3), readings (table by connector), connectors (raw traces)
-- The 'maxContent' across connectors reflects the tank's stored amount.
local function read_connectors(tank)
  local connectors = tank:getPipeConnectors() or {}
  local readings = {}
  local maxContent = 0

  for i = 1, #connectors do
    local c = connectors[i]
    local r = {
      content     = c.fluidBoxContent or 0,
      flowFill    = c.fluidBoxFlowFill or 0,
      flowDrain   = c.fluidBoxFlowDrain or 0,
      flowThrough = c.fluidBoxFlowThrough or 0,
      height      = c.fluidBoxHeight or 0,
    }
    readings[i] = r
    if r.content > maxContent then maxContent = r.content end
  end

  return maxContent, readings, connectors
end

-- Flushes the pipe network attached to the given tank (via its first connector).
-- NOTE: This empties the *entire pipe network* on that connector, not just the tank.
local function flush_network_for_tank(tank)
  local connectors = tank:getPipeConnectors()
  local c = connectors and connectors[1]
  if c then
    print(("Flushing network on tank %s ..."):format(tank.nick ~= "" and tank.nick or tank.id))
    c:flushPipeNetwork()
  else
    print("  (No pipe connectors found on tank; nothing to flush.)")
  end
end

-- === Main loop ===
print("AutoFlush IFB running. Watching nick = '" .. TANK_NICK_FILTER .. "' ...")
print(("Capacity=%.0f m^3, Threshold=%.0f m^3 (%.0f%%), Poll=%ds")
      :format(CAPACITY_M3, CAPACITY_M3*THRESHOLD_PCT, THRESHOLD_PCT*100, POLL_SECONDS))

while true do
  for i = 1, #tanks do
    local tank = tanks[i]
    local name = (tank.nick ~= "" and tank.nick or tank.id)

    local fill_m3, readings, connectors = read_connectors(tank)
    local pct = (CAPACITY_M3 > 0) and (fill_m3 / CAPACITY_M3) or 0

    -- Per-poll summary
    print(("Check: %s  %.1f/%.0f m^3  (%.1f%%)")
          :format(name, fill_m3, CAPACITY_M3, pct * 100))

    -- Optional per-connector debug
    if DEBUG_CONNECTORS then
      for idx, r in ipairs(readings) do
        print((
          "  conn %d: content=%.1f  flowFill=%.1f  flowDrain=%.1f  flowThrough=%.1f  height=%.1f"
        ):format(idx, r.content, r.flowFill, r.flowDrain, r.flowThrough, r.height))
      end
    end

    -- Threshold check & flush
    if pct >= THRESHOLD_PCT then
      print(("Threshold reached: %s  %.0f/%.0f m^3 (%.0f%%)")
            :format(name, fill_m3, CAPACITY_M3, pct*100))
      flush_network_for_tank(tank)
      sleep(POST_FLUSH_COOLDOWN) -- cooldown after flush to avoid repeated flushes
    end
  end

  sleep(POLL_SECONDS)
end

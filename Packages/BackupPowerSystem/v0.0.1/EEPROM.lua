---
--- Created by Rostriano
--- Date: 2024-07-04
---

--- The Ficsit Backup Power System manages backup power - sort of like a UPS but too slow to guarantee uninterrupted power.
---
--- To use the system, you will need one or more batteries, plus two regular switches and of course a computer.
--  The switches must be hooked up as drawn below:
---
---  mains power    load     battery
---       |           |         |
---       |           |         |
---       +-----/ ----+----/ ---+
---
---           mains      battery
---          switch      switch
---
--- The mains switch must have the following nick: "BPS mainsSwitch"
--- The battery switch must have the following nick: "BPS batterySwitch"
---
--- The computer needs to have a network connection to both switches and a battery.
---
--- To guarantee grid stability, Ficsit advises to have a minimum of one battery connected
--- directly to the grid. This ensures there will be enough power available for the limited
--- amount of time it takes for your Backup Power management system to flip its switches.

local POLL_INTERVAL <const> = 5
local CATEGORY <const> = "BPS"

local MODE <const> = { NORMAL="1", DISCHARGING="2", CHARGING="3", OFF="4", TRIPPED_FUSE="5" }


---Find and return a table of all the NetworkComponent proxies that are of the given class[es]
---@param class any Class name or table (of tables) of class names
---@param boolean Return only one
---@return table | nil | proxy: indexed table of all NetworkComponents found
function getComponentsByClass( class, getOne )
    local results = {}

    if ( getOne == nil ) then
        getOne = false
    end

    if type( class ) == "table" then

        for _, c in pairs( class ) do
            local proxies = getComponentsByClass( c, getOne )
            if not getOne then
                tableConcat( results, proxies )
            else
                if( proxies ~= nil ) then
                    return proxies
                end
            end
        end

    elseif type( class ) == "string" then

        local ctype = classes[ class ]
        if ctype ~= nil then
            local comps = component.findComponent( ctype )
            for _, c in pairs( comps ) do
                local proxy = component.proxy( c )
                if getOne and proxy ~= nil then
                    return proxy
                elseif not tableHasValue( results, proxy ) then
                    table.insert( results, proxy )
                end
            end
        end

    end

    if ( getOne ) then
        return {}
    end

    return results
end


local battery = getComponentsByClass( { "PowerStorage" }, true )
    or computer.panic( "Battery not found" )
local batterySwitch = component.proxy( component.findComponent(CATEGORY .. " batterySwitch" )[1] )
    or computer.panic( "Battery switch not found" )
local mainsSwitch = component.proxy( component.findComponent(CATEGORY .. " mainsSwitch" )[1] )
    or computer.panic( "Mains switch not found" )
local connectors = mainsSwitch:getPowerConnectors()
    or computer.panic( "Mains switch power connectors not found" )
local circuit1, circuit2, currMode

function hasTrippedFuse( circuits )
  for _, circuit in pairs( circuits ) do
      if circuit.isFuesed then
          return true
      end
  end

  return false
end

function getGridSurplus( gridCircuit )
  gridProduction = ( gridCircuit and gridCircuit.production ) or 0
  gridConsumption = gridCircuit.consumption or 0
  gridSurplus = gridProduction - gridConsumption

  print( "production, consumption, surplus:", gridProduction, gridConsumption, gridSurplus )

  return gridSurplus
end

-- Set switches to default settings and then enter loop to figure out where we stand
batterySwitch:setIsSwitchOn( false )
mainsSwitch:setIsSwitchOn( true )

while( true ) do

  -- We get the circuits inside the loop so that we can see a wire being attached
  circuit1, circuit2 = connectors[1]:getCircuit(), connectors[2]:getCircuit()

  -- Determine if the grid has power
  if circuit1 and circuit1.production > 0 then
    gridCircuit = circuit1
  else
    gridCircuit = circuit2
  end

  -- Determine what to do
  if  gridCircuit ~= nil and gridCircuit.production > 0 then
    if battery.powerStore == 100 and not hasTrippedFuse( { circuit1, circuit2 } ) then

      -- We got mains power and the battery doesn't need to charge, all is well
      if currMode ~= MODE.NORMAL then
        print( "Switching to normal mode" )
        batterySwitch:setIsSwitchOn( false )
        mainsSwitch:setIsSwitchOn( true )
        currMode = MODE.NORMAL
      end
    elseif
      not hasTrippedFuse( { circuit1, circuit2 } )
      and getGridSurplus( gridCircuit ) > 0 -- Make sure we won't accidentally discharge the battery
    then
      -- We got mains power; charge battery
      if currMode ~= MODE.CHARGING then
        print( "Charging battery" )
        mainsSwitch:setIsSwitchOn( true )
        batterySwitch:setIsSwitchOn( true )
        currMode = MODE.CHARGING
      end
      print( "Battery charge", battery.powerStore, "MWh" )
    end
  else -- gridCircuit.production == 0
    -- Mains power down; run on battery
    if currMode ~= MODE.DISCHARGING then
      print( "Running on battery" )
      currMode = MODE.DISCHARGING
    end
    mainsSwitch:setIsSwitchOn( false )
    batterySwitch:setIsSwitchOn( true )
    print( "Battery charge", battery.powerStore, "MWh" )
  end

  ::continue::
  event.pull( POLL_INTERVAL )
end

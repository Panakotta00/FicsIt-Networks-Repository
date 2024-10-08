---
--- Ficsit Backup Power Supply
---
--- Created by Rostriano, with code by 1000101
--- Date: 2024-07-04
---

local POLL_INTERVAL <const> = 5
local CATEGORY <const> = "BPS"

local MODE <const> = { NORMAL="1", DISCHARGING="2", CHARGING="3" }


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


local powerStorage = getComponentsByClass( { "PowerStorage" }, true )
    or computer.panic( "Power storage not found" )
local powerStorageSwitch = component.proxy( component.findComponent(CATEGORY .. " powerStorageSwitch" )[1] )
    or computer.panic( "Power storage switch not found" )
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
powerStorageSwitch:setIsSwitchOn( false )
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
    if powerStorage.powerStore == 100 and not hasTrippedFuse( { circuit1, circuit2 } ) then

      -- We got mains power and the power storage doesn't need to charge, all is well
      if currMode ~= MODE.NORMAL then
        print( "Switching to normal mode" )
        powerStorageSwitch:setIsSwitchOn( false )
        mainsSwitch:setIsSwitchOn( true )
        currMode = MODE.NORMAL
      end
    elseif
      not hasTrippedFuse( { circuit1, circuit2 } )
      and getGridSurplus( gridCircuit ) > 0 -- Make sure we won't accidentally discharge the power storage
    then
      -- We got mains power; charge power storage
      if currMode ~= MODE.CHARGING then
        print( "Charging power storage" )
        mainsSwitch:setIsSwitchOn( true )
        powerStorageSwitch:setIsSwitchOn( true )
        currMode = MODE.CHARGING
      end
      print( "Power storage charge", powerStorage.powerStore, "MWh" )
    end
  else -- gridCircuit.production == 0
    -- Mains power down; run on power storage
    if currMode ~= MODE.DISCHARGING then
      print( "Running on power storage" )
      currMode = MODE.DISCHARGING
    end
    mainsSwitch:setIsSwitchOn( false )
    powerStorageSwitch:setIsSwitchOn( true )
    print( "Power storage charge", powerStorage.powerStore, "MWh" )
  end

  ::continue::
  event.pull( POLL_INTERVAL )
end

---
--- Ficsit multi-screen and multi-timezone clock
---
--- Created by Rostriano
--- Date: 2024-09-07
---
--- Based on work by 1000101 and Aider
---

-- These dimensions are optimal for a screen that has the smallest height possible and is one click wider than the minimum possible width
SCREEN_SIZE = { x = 7, y = 1 }

-- Adds the contents of t2 to t1
function tableConcat( t1, t2 )
    for i=1, #t2 do
       t1[#t1+1] = t2[i]
    end
    return t1
end

---Can the given value be found in a table of { key, values } ?
---@param t table
---@param value any
---@return boolean
function tableHasValue( t, value )
    if t == nil or value == nil then
        return false
    end

    for _,v in pairs( t ) do
        if v == value then
            return true
        end
    end

    return false
end

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

---Find and return a table of all the NetworkComponent proxies that are of the given class[es] and contain the given nick parts
---@param class any Class name or table (of tables) of class names
---@param class nickParts Nick or parts of a nick that we want to see
---@return table: indexed table of all NetworkComponents found
function getComponentsByClassAndNick( class, nickParts )
    if type( nickParts ) == 'string' then
        nickParts = { nickParts }
    end

    local classComponents = getComponentsByClass( class )
    local results = {}

    for _, component in pairs( classComponents ) do
        for _, nickPart in pairs( nickParts ) do
            if component.nick:find( nickPart, 1, true ) == nil then
                goto nextComponent
            end
        end

        table.insert( results, component )

        ::nextComponent::
    end

    return results
end

---Decodes a table of settings stored in a string.
---Settings should be encoded as: key1="value1" key2="value2" ...
---@param str string Tokenized string to decode
---@param lowerKeys? boolean Force the keys to be lowercase, otherwise any cAmElCaSInG is preserved and it will be usercode responsibility to deal with it
---@return table table {key=field,value=value*} *value will not be quoted despite the requirement to enclose the value in double quotes in the string
function settingsFromString( str, lowerKeys )
    lowerKeys = lowerKeys or false
    local results = {}
    if str == nil or type( str ) ~= "string" then return results end
    for key, value in string.gmatch( str, '(%w+)=(%b"")' ) do
        if lowerKeys then key = string.lower( key ) end
        results[ key ] = string.sub( value, 2, string.len( value ) - 1 )
    end
    return results
end

---Reads the table of settings stored in a Network Components nickname.  This function does not apply the setings, this merely reads the string and turns it into a {key, value} table.
---Settings should be encoded as: key1="value1" key2="value2" ...
---@param proxy userdata NetworkComponent proxy
---@param lowerKeys? boolean Force the keys to be lowercase, otherwise any cAmElCaSInG is preserved and it will be usercode responsibility to deal with it
---@return table table {key=field,value=value*} *value will not be quoted despite the requirement to enclose the value in double quotes in the nick
function settingsFromComponentNickname( proxy, lowerKeys )
    if proxy == nil then return nil end
    return settingsFromString( proxy[ "nick" ], lowerKeys )
end

function initFromConfig()
  local gpus = computer.getPCIDevices( classes.GPU_T1_C )
  if gpus == nil then
    error( "No GPU T1 found" )
  end

  local computerSettings = settingsFromComponentNickname( computer.getInstance() )
  local screenNick = computerSettings.screen or "clock"
  local screens = getComponentsByClassAndNick( {
    "ModuleScreen_C",
    "Build_Screen_C",
  }, screenNick )
  if #screens == 0 then
    error( "No screens found with the '" .. screenNick .. "' nick" )
  end

  if #gpus < #screens then
    computer.panic( 'Not enough GPUs to drive all the screens: ' .. #gpus .. ' gpus vs ' .. #screens .. ' screens' )
  end

  local screenDefinitions =  {}
  for i, screen in pairs( screens ) do
    local timeDisplayFunction = displayM2CT
    local settings = settingsFromComponentNickname( screen, true )
    if rawget( settings, 'tz' ) == 'UTC' then
      timeDisplayFunction = displayUTC
    end

    table.insert( screenDefinitions, {
      gpu = gpus[i],
      screen = screen,
      display = timeDisplayFunction,
    })
  end

  return screenDefinitions
end

function initScreens( screenDefinitions )
  for _, screenDefinition in pairs( screenDefinitions ) do
    screenDefinition.gpu:bindScreen( screenDefinition.screen )
    screenDefinition.gpu:setSize( SCREEN_SIZE.x, SCREEN_SIZE.y )
    screenDefinition.gpu:setForeground( 1, 1, 1, 1 )
    screenDefinition.gpu:setBackground( 0, 0, 0, 0 )
  end
end

function clearScreens( screenDefinitions )
  for _, screenDefinition in pairs( screenDefinitions ) do
    screenDefinition.gpu:setSize( SCREEN_SIZE.x, SCREEN_SIZE.y )
    screenDefinition.gpu:fill( 0, 0, SCREEN_SIZE.x, SCREEN_SIZE.y, " " )
  end
end

function updateScreens( screenDefinitions )
  for _, screenDefinition in pairs( screenDefinitions ) do
    screenDefinition:display()
  end
end

function formatTime( timestamp )
  return string.format(
    "%2d:%02d",
    math.floor( timestamp / 3600 ) % 24,
    math.floor( timestamp / 60 ) % 60
  )
end

function displayM2CT( screenDefinition )
  screenDefinition.gpu:setText( 1, 0, formatTime( computer.time() ) )
  screenDefinition.gpu:flush()
end

function displayUTC( screenDefinition )
  screenDefinition.gpu:setText( 1, 0, formatTime( computer.magicTime()) )
  screenDefinition.gpu:flush()
end

function init()
  local screenDefinitions = initFromConfig()
  clearScreens( screenDefinitions )
  initScreens( screenDefinitions )

  return screenDefinitions
end

local screenDefinitions = init()
while true do
  event.pull(0.8)
  updateScreens( screenDefinitions )
end

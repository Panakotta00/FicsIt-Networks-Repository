--- PowerMonitor
---
--- Created by Rostriano
--- Date: 2024-08-08
---

--------------------------------------------------------------------------------
-- Utility functions
--------------------------------------------------------------------------------

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


--------------------------------------------------------------------------------
-- Color
--------------------------------------------------------------------------------
Color = {
    r = 0.0,
    g = 0.0,
    b = 0.0,
    a = 0.0,
    pattern = '{r=%1.6f,g=%1.6f,b=%1.6f,a=%1.6f}',
}
Color.__index = Color

---Create a new Color and return it or nil on invalid input
---@param r number
---@param g number
---@param b number
---@param a number
---@return Color
function Color.new( r, g, b, a )
    if r == nil or type( r ) ~= "number" then return nil end
    if g == nil or type( g ) ~= "number" then return nil end
    if b == nil or type( b ) ~= "number" then return nil end
    if a == nil or type( a ) ~= "number" then return nil end
    local o = {
        r = r,
        g = g,
        b = b,
        a = a,
    }
    setmetatable( o, { __index = Color } )
    return o
end

Color.BLACK             = Color.new( 0.000, 0.000, 0.000, 1.0 )
Color.WHITE             = Color.new( 1.000, 1.000, 1.000, 1.0 )
Color.GREY_0750         = Color.new( 0.750, 0.750, 0.750, 1.0 )
Color.GREY_0500         = Color.new( 0.500, 0.500, 0.500, 1.0 )
Color.GREY_0250         = Color.new( 0.250, 0.250, 0.250, 1.0 )
Color.GREY_0125         = Color.new( 0.125, 0.125, 0.125, 1.0 )

Color.RED               = Color.new( 1.000, 0.000, 0.000, 1.0 )
Color.GREEN             = Color.new( 0.000, 1.000, 0.000, 1.0 )
Color.GREEN_0750        = Color.new( 0.000, 0.750, 0.000, 1.0 )
Color.GREEN_0500        = Color.new( 0.000, 0.500, 0.000, 1.0 )
Color.BLUE              = Color.new( 0.000, 0.000, 1.000, 1.0 )

Color.FICSIT_ORANGE     = Color.new( 1.000, 0.550, 0.200, 1.0 )


--------------------------------------------------------------------------------
-- Vector 2d
--------------------------------------------------------------------------------
Vector2d = {
    x = 0,
    y = 0,
    pattern = '{x=%d,y=%d}',
}
Vector2d.__index = Vector2d

---Create a new Vector2d and return it
---@param x integer
---@param y integer
---@return Vector2d
function Vector2d.new( x, y )
    if x == nil or type( x ) ~= "number" then return nil end
    if y == nil or type( y ) ~= "number" then return nil end
    local o = { x = math.floor( x ), y = math.floor( y ) }
    setmetatable( o, { __index = Vector2d } )
    return o
end


--------------------------------------------------------------------------------
-- SizeLimitedList
--------------------------------------------------------------------------------
SizeLimitedList = {}
SizeLimitedList.__index = SizeLimitedList

function SizeLimitedList.new( maxSize )
  local self = setmetatable( {
    first = 0,
    maxSize = 10,
    currSize = 0,
    items = {},

    maxVal = nil,
    minVal = nil,


  }, SizeLimitedList )
  if maxSize ~= nil then
      self.maxSize = maxSize
  end

  return self
end

function SizeLimitedList:setSize( newSize )
    if self.currSize > newSize then
        local shrinkBy = self.currSize - newSize
        local newFirst = self.first + shrinkBy

        while self.first < newFirst do
            self.items[ self.first ] = nil
            self.first = self.first + 1
        end
    end

    self.currSize = math.min( self.currSize, newSize )
    self.maxSize = newSize
end

function SizeLimitedList:getSize()
    return self.currSize
end

function SizeLimitedList:getMaxSize()
    return self.maxSize
end

function SizeLimitedList:add( item )
    self.items[ self.first + self.currSize ] = item

    if self.currSize < self.maxSize then
        self.currSize = self.currSize + 1
    else
        self.items[ self.first ] = nil
        self.first = self.first + 1
    end

    self:updateMinMaxVals()
end

function SizeLimitedList:getMinVal( default )
    return self.minVal or default
end

function SizeLimitedList:getMaxVal( default )
    return self.maxVal or default
end

function SizeLimitedList:iterate( f )
    local index = self.first
    local sentinel = self.first + self.currSize

    while index < sentinel do
        f( self.items[ index ] )
        index = index + 1
    end
end

function SizeLimitedList:updateMinMaxVals()
    self.minVal = nil
    self.maxVal = nil

    self:iterate(
        function( currVal )
            if self.minVal == nil then
                self.minVal = currVal
            else
                self.minVal = math.min( self.minVal, currVal )
            end

            if self.maxVal == nil then
                self.maxVal = currVal
            else
                self.maxVal = math.max( self.maxVal, currVal )
            end
        end
    )
end


--------------------------------------------------------------------------------
-- ScreenElement
--------------------------------------------------------------------------------
--[[
     Offers the regular GPU T2 drawing commands, translating
     the coordinates to a screen location.

     This serves as the base for a graphics library, allowing
     multiple elements to be added that will (re)draw when
     this Element is being (re)drawn
]]
ScreenElement = {
    gpu = nil,
    position = nil,
    dimensions = nil,
    subElements = {},

    -- Helper functions at the bottom of this script
    reposition = nil,
}

function ScreenElement:new( o )
    o = o or {}
    self.__index = self
    setmetatable( o, self )

    return o
end

function ScreenElement:init( gpu, position, dimensions )
    self.gpu = gpu
    self.position = position
    self.dimensions = dimensions
end

function ScreenElement:addElement( e )
    if e == nil then
        return
    end

    table.insert( self.subElements, e )
end

-- Draw the element and all sub elements that have been added
function ScreenElement:draw()
    print( "Repainting" )
    for _, element in pairs(self.subElements) do
        element:draw()
    end
end

function ScreenElement:flush()
    computer.error( 'ScreenElement:flush() should not be called; call draw(), then do a gpu:flush()' )
end

function ScreenElement:measureText( Text, Size, bMonospace )
    return self.gpu:measureText( Text, Size, bMonospace )
end

--- Draws some Text at the given position (top left corner of the text), text, size, color and rotation.
---@param position Vector2D @The position of the top left corner of the text.
---@param text string @The text to draw.
---@param size number @The font size used.
---@param color Color @The color of the text.
---@param monospace boolean @True if a monospace font should be used.
function ScreenElement:drawText( position, text, size, color, monospace )
    self.gpu:drawText(
        self:reposition( position ),
        text,
        size,
        color,
        monospace
    )
end

--- Draws a Rectangle with the upper left corner at the given local position, size, color and rotation around the upper left corner.
---@param position Vector2D @The local position of the upper left corner of the rectangle.
---@param size Vector2D @The size of the rectangle.
---@param color Color @The color of the rectangle.
---@param image string @If not empty string, should be image reference that should be placed inside the rectangle.
---@param rotation number @The rotation of the rectangle around the upper left corner in degrees.
function ScreenElement:drawRect( position, size, color, image, rotation )
    self.gpu:drawRect(
        self:reposition( position ),
        size,
        color,
        image,
        rotation
    )
end

--- Draws connected lines through all given points with the given thickness and color.
---@param points Vector2D[] @The local points that get connected by lines one after the other.
---@param thickness number @The thickness of the lines.
---@param color Color @The color of the lines.
function ScreenElement:drawLines( points, thickness, color )
    if #points < 2 then
       return
    end

    local newPoints = {}

    for _, currPoint in pairs( points ) do
        table.insert( newPoints, self:reposition( currPoint ) )
    end

    self.gpu:drawLines(
        newPoints,
        thickness,
        color
    )
end

function ScreenElement:reposition( vector )
    return Vector2d.new(
        self.position.x + vector.x,
        self.position.y + vector.y
    )
end


--------------------------------------------------------------------------------
-- Plotter
--------------------------------------------------------------------------------
Plotter = ScreenElement:new()

Plotter.__index = Plotter
Plotter.graph = nil -- The graph this plotter belongs to
Plotter.maxVal = nil
Plotter.scaleFactorX = nil
Plotter.color = Color.GREY_0500
Plotter.lineThickness = 10
Plotter.dataSource = {}

function Plotter.new( o )
    local plotter = setmetatable( o or {}, Plotter )

    if o ~= nil and rawget( o, 'dataSource' ) ~= nil then
        plotter:setDataSource( o.dataSource )
    end

    return plotter
end

function Plotter:setDataSource( dataSource )
    self.dataSource = dataSource
    self.scaleFactorX = self.graph.dimensions.x / ( self.dataSource:getMaxSize() - 1 )
end

function Plotter:setColor( color )
    self.color = color
end

function Plotter:setLineThickness( lineThickness )
    self.lineThickness = lineThickness
end

function Plotter:draw()
    local i = 0
    local points = {}
    self.dataSource:iterate(
        function( currVal )
            local xPos = ( i + self.dataSource.maxSize - self.dataSource.currSize ) * self.scaleFactorX
            local yPos = self.graph.dimensions.y - currVal * self.graph.scaleFactorY

            local position = Vector2d.new( xPos, yPos )
            table.insert( points, position )
            i = i + 1
        end
    )

    self:drawLines( points, self.lineThickness, self.color )
end


--------------------------------------------------------------------------------
-- Graph
--------------------------------------------------------------------------------
Graph = ScreenElement:new()

Graph.__index = Graph
Graph.scaleFactorY = nil
Graph.maxVal = nil
Graph.dimensions = nil
Graph.scaleMarginFactor = 0.2
Graph.dataSources = {}
Graph.plotters = {}

function Graph.new()
    return setmetatable( {}, Graph )
end

function Graph:addPlotter( name, config )
    local plotter = Plotter.new()
    self.plotters[ name ] = plotter

    if config ~= nil then
        self:configurePlotter( name, config )
    end
end

function Graph:configurePlotter( name, config )
    local plotter = self.plotters[ name ]
    for k, v in pairs( config ) do
        plotter[ k ] = v
    end
    plotter.graph = self

    if rawget( plotter, 'dataSource' ) ~= nil then
        plotter:setDataSource( config.dataSource )
        table.insert( self.dataSources, config.dataSource or {} )
    end
end

function Graph:setMaxVal( maxVal )
    self.maxVal = maxVal
    self.scaleFactorY = self.dimensions.y / maxVal
end

function Graph:setDimensions( dimensions )
    self.dimensions = dimensions

    for _,currItem in ipairs( self.dataSources ) do
        currItem.dimensions = dimensions
    end
end

function Graph:draw()
    if self.maxVal == nil then
        self:autoResize()
    end

    for _, plotter in pairs( self.plotters ) do
        plotter:draw()
    end
end

function Graph:autoResize()
    local maxVal = self:getMaxVal()

    if self.scaleFactorY == nil then
        return self:initScaleFactors( maxVal )
    end

    local maxDisplayableVal = self.dimensions.y / ( self.scaleFactorY or 0.00000000001 )
    if
        maxDisplayableVal < maxVal or
        maxDisplayableVal * self.scaleMarginFactor > maxVal
    then
        self:initScaleFactors( maxVal )
    end
end

function Graph:initScaleFactors( maxVal )
    if maxVal == nil then
        maxVal = 0.00000000001
    end

    self.scaleFactorY = self.dimensions.y / ( maxVal * ( 1 + self.scaleMarginFactor ) )
end

function Graph:getMaxVal()
    local maxVal = nil
    for _,currItem in ipairs( self.dataSources ) do
        maxVal = math.max( maxVal or currItem:getMaxVal(), currItem:getMaxVal() )
    end
    return maxVal
end


--------------------------------------------------------------------------------
-- Footer
--------------------------------------------------------------------------------
Footer = ScreenElement:new()
Footer.fontSize = 50
Footer.textColor = Color.GREY_0750
Footer.textVerticalOffset = -22

function Footer:draw()
    self:drawRect( Vector2d.new( 100,50 ), Vector2d.new( 50,50 ), self.colors.consumption, nil, nil )
    self:drawText( Vector2d.new( 200,50+self.textVerticalOffset ), self._getLabel( "Consumption", self.values.consumption), self.fontSize, self.textColor)

    self:drawRect( Vector2d.new( 100,150 ), Vector2d.new( 50,50 ), self.colors.production, nil, nil )
    self:drawText( Vector2d.new( 200,150+self.textVerticalOffset ), self._getLabel( "Production", self.values.production), self.fontSize, self.textColor)

    self:drawRect( Vector2d.new( 1100,50 ), Vector2d.new( 50,50 ), self.colors.maxConsumption, nil, nil )
    self:drawText( Vector2d.new( 1200,50+self.textVerticalOffset ), self._getLabel( "Max. consumption", self.values.maxPowerConsumption), self.fontSize, self.textColor)

    self:drawRect( Vector2d.new( 1100,150 ), Vector2d.new( 50,50 ), self.colors.capacity, nil, nil )
    self:drawText( Vector2d.new( 1200,150+self.textVerticalOffset ), self._getLabel( "Production capacity", self.values.capacity), self.fontSize, self.textColor)
end

function Footer:setValues(values)
    self.values = values
end

function Footer._getLabel( text, value )
    if value == nil then
        value = 'NaN'
    else
        value = string.format( '%.1f', value )
    end

    return text .. ' ' .. value .. ' MW'
end


--------------------------------------------------------------------------------
-- BatteryInfo
--------------------------------------------------------------------------------
BatteryInfo = ScreenElement:new()
BatteryInfo.line = 0
BatteryInfo.lineHeight = 60
BatteryInfo.fontSize = 35
BatteryInfo.textColor = Color.GREY_0750
BatteryInfo.dataFontSize = 50
BatteryInfo.dataColor = Color.WHITE
BatteryInfo.colorBad = Color.RED
BatteryInfo.colorGood = Color.new( 0.000, 1.000, 0.000, 1.0 )

function BatteryInfo:draw()
    -- We get the circuit inside the loop so that stuff doesn't crash whenever a wire is detached
    local circuit = self.connector:getCircuit()

    if not circuit.hasBatteries then
        return
    end

    local batteryCapacity = ( circuit and circuit.batteryCapacity ) or 0
    local batteryStore = ( circuit and circuit.batteryStore ) or 0
    local batteryStorePercent = ( circuit and 100 * circuit.batteryStorePercent ) or 0
    local batteryTimeUntilFull = ( circuit and circuit.batteryTimeUntilFull ) or 0
    local batteryTimeUntilEmpty = ( circuit and circuit.batteryTimeUntilEmpty ) or 0
    local batteryIn = ( circuit and circuit.batteryIn ) or 0
    local batteryOut = ( circuit and circuit.batteryOut ) or 0

    self.line = 0

    self:print( 'Stored' )
    self:print( string.format( '%.1f %%', batteryStorePercent ), self.dataFontSize, self.percentageColor( batteryStorePercent ) )
    self.line = self.line + 2

    self:print( 'Charge' )
    if( batteryStorePercent == 100 ) then
        self:print( string.format( '%.1f MWh', batteryStore ), self.dataFontSize, self.percentageColor( batteryStorePercent ) )
    else
        self:print( string.format( '%.1f / %.1f MWh', batteryStore, batteryCapacity ), self.dataFontSize, self.percentageColor( batteryStorePercent ) )
    end
    self.line = self.line + 2

    if batteryOut > 0 then
        self:print( 'Discharge rate' )
        self:print( string.format( '%.1f MW', batteryOut ), self.dataFontSize, self.colorBad )
        self.line = self.line + 2
    elseif batteryIn > 0 then
        self:print( 'Charge rate' )
        self:print( string.format( '%.1f MW', batteryIn ), self.dataFontSize, self.colorGood )
        self.line = self.line + 2
    end

    if batteryTimeUntilEmpty > 0 then
        self:print( 'Time until empty ' )
        self:print( self.formatTime( batteryTimeUntilEmpty ), self.dataFontSize, self.colorBad )
        self.line = self.line + 2
    elseif batteryTimeUntilFull > 0 then
        self:print( 'Time until full' )
        self:print( self.formatTime( batteryTimeUntilFull ), self.dataFontSize, self.colorGood )
        self.line = self.line + 2
    end
end

function BatteryInfo:print( text, size, color )
    self.line = self.line + 1

    local yPos = self.line * self.lineHeight

    size = size or self.fontSize
    color = color or self.textColor
    self:drawText( Vector2d.new( 40,yPos ), text, size, color)
end

function BatteryInfo.formatTime( seconds )
    return string.format(
      "%02d:%02d:%02d",
      math.floor( seconds / 3600 ) % 24,
      math.floor( seconds / 60 ) % 60,
      math.floor( seconds % 60 )
    )
end

function BatteryInfo.percentageColor( percentage )
    if percentage < 33 then
        return Color.RED
    elseif percentage < 80 then
        return Color.FICSIT_ORANGE
    elseif percentage < 100 then
        return Color.GREEN
    else
        return Color.GREY_0750
    end
end


--------------------------------------------------------------------------------
-- PowerMonitor
--------------------------------------------------------------------------------
PowerMonitor = {
    connector = nil,
    gpu = nil,
    pollInterval = 1,

    productionList = nil,
    consumptionList = nil,
    maxPowerConsumptionList = nil,

    powerGraph = nil,
    footer = nil,

    colors = {
        consumption = Color.FICSIT_ORANGE,
        capacity = Color.GREY_0500,
        production = Color.GREY_0750,
        maxConsumption = Color.new( 0.050, 0.500, 0.700, 1.0 ),
    },

    graphWidth = 2300,
    graphHeight = 1550,
}

function PowerMonitor:init( power, gpu )
    print( "\nInitialising PowerMonitor\n" )

    self.gpu = gpu
    local connectors = power:getPowerConnectors()
    if #connectors == 0 then
        computer.panic( 'The power interface has no connectors, cannot continue' )
    end
    self.connector = connectors[1]

    self:initLists()

    -- Screen size is 3000x1800
    self.powerGraph = Graph.new()
    self.powerGraph:setDimensions( Vector2d.new( self.graphWidth,self.graphHeight ) )
    self.powerGraph:addPlotter(
        'consumption',
        {
            gpu = self.gpu,
            position = Vector2d.new( 0,0 ),
            color = self.colors.consumption,
            dataSource = self.consumptionList,
        }
    )
    self.powerGraph:addPlotter(
        'capacity',
        {
            gpu = self.gpu,
            position = Vector2d.new( 0,0 ),
            color = self.colors.capacity,
            dataSource = self.capacityList,
        }
    )
    self.powerGraph:addPlotter(
        'production',
        {
            gpu = self.gpu,
            position = Vector2d.new( 0,0 ),
            color = self.colors.production,
            dataSource = self.productionList,
        }
    )
    self.powerGraph:addPlotter(
        'maxConsumption',
        {
            gpu = self.gpu,
            position = Vector2d.new( 0,0 ),
            color = self.colors.maxConsumption,
            dataSource = self.maxPowerConsumptionList,
        }
    )

    self.batteryInfo = BatteryInfo:new({ connector = self.connector })
    self.batteryInfo:init( self.gpu, Vector2d.new( self.graphWidth,0 ), Vector2d.new( 3000-self.graphWidth,1800  ) )

    self.footer = Footer:new({colors = self.colors})
    self.footer:init( self.gpu, Vector2d.new( 0, self.graphHeight ), Vector2d.new( self.graphWidth, 1000 ) )
end

function PowerMonitor:run()
    print( "PowerMonitor running")

    while true do
        self:collectData()

        -- Paint background
        -- self.gpu:drawRect( Vector2d.new(0,0), Vector2d.new(3000,1800), Color.WHITE, nil, nil )

        self.gpu:drawLines(
            { Vector2d.new( 0,self.graphHeight ), Vector2d.new( self.graphWidth,self.graphHeight ) },
            5,
            Color.GREY_0500
        )
        self.gpu:drawLines(
            { Vector2d.new( self.graphWidth,0 ), Vector2d.new( self.graphWidth,self.graphHeight ) },
            5,
            Color.GREY_0500
        )

        self.powerGraph:draw()

        self.footer:setValues({
            production = self.production,
            capacity = self.capacity,
            consumption = self.consumption,
            maxPowerConsumption = self.maxPowerConsumption,
        })
        self.footer:draw()

        self.batteryInfo:draw()

        self.gpu:flush()
        event.pull( self.pollInterval )
    end
end

function PowerMonitor:initLists()
    self.productionList = SizeLimitedList.new( 100 )
    self.capacityList = SizeLimitedList.new( 100 )
    self.consumptionList = SizeLimitedList.new( 100 )
    self.maxPowerConsumptionList = SizeLimitedList.new( 100 )
end

function PowerMonitor:collectData()
    -- We get the circuit inside the loop so that stuff doesn't crash whenever a wire is detached
    local circuit = self.connector:getCircuit()

    self.production = ( circuit and circuit.production ) or 0
    self.capacity = ( circuit and circuit.capacity ) or 0
    self.consumption = ( circuit and circuit.consumption ) or 0
    self.maxPowerConsumption = ( circuit and circuit.maxPowerConsumption ) or 0

    self.productionList:add( self.production )
    self.capacityList:add( self.capacity )
    self.consumptionList:add( self.consumption )
    self.maxPowerConsumptionList:add( self.maxPowerConsumption )
end

--------------------------------------------------------------------------------

local power = getComponentsByClass( {
    "FGBuildablePowerPole", -- Power poles and wall outlets
    "CircuitSwitch",
    "Build_PriorityPowerSwitch_C",
    "PowerStorage",
} )
if #power == 0 then
    computer.panic( "No power pole or wall outlet hooked up; nothing to monitor" )
end
power = power[1]

local gpu = computer.getPCIDevices( classes.GPU_T2_C )[1]
if gpu == nil then
    computer.panic( "No GPU T2 found. Cannot continue." )
end

local computerSettings = settingsFromComponentNickname( computer.getInstance() )
local screens = getComponentsByClassAndNick( {
    "ModuleScreen_C",
    "Build_Screen_C",
}, computerSettings.screen or '' )
if #screens == 0 then
    computer.panic( "No screen found. Cannot continue." )
end

gpu:bindScreen( screens[1] )

screenSize = gpu:getScreenSize()
print( 'Screen resolution: ' .. screenSize.x .. 'x' .. screenSize.y )

PowerMonitor:init( power, gpu )
PowerMonitor:run()

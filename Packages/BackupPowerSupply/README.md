# The Backup Power Supply
The Ficsit Backup Power Supply manages backup power - sort of like a UPS but too slow to
guarantee uninterrupted power. Or is it?

## Setup
To use the Ficsit BPS, you will need one or more power storages, plus two power switches and of course a computer.
The switches must be hooked up as shown below:

     mains power    load    power storage
          |           |           |
          |           |           |
          +-----/ ----+----/ -----+

              mains     power storage
             switch        switch


A demonstration setup is shown below:

![Example setup of the Ficsit Backup Power Supply](preview1.jpg "Example setup of the Ficsit BPS")
<sub>Example setup of the Ficsit Backup Power Supply</sub>

Please observe the following:
- the mains switch must have the following nick: "BPS mainsSwitch"
- the power storage switch must have the following nick: "BPS powerStorageSwitch"
- the computer needs to have a network connection to both switches and a power storage.

## Operation and functioning
Once operational, the Ficsit Backup Power Supply will not need any attention from the employee.

The Ficsit BPS will monitor the grid and the connected power storage(s). When the overall load on the grid exceeds the production, it will disconnect itself from the grid and engage the power storage(s) so you can continue operating from the power storage(s) as long as there is any charge left in the power storage(s). Then, when the grid is balanced again, it will start charging the power storage(s).

![Normal operation](preview2.jpg "The Ficsit BPS during Normal operation")
<sub>The Ficsit BPS during Normal operation</sub>

![Charging](preview3.jpg "The Ficsit BPS while Charging")
<sub>The Ficsit BPS while Charging</sub>

![Providing backup power](preview4.jpg "The ficsit BPS providing backup power")
<sub>The ficsit BPS providing backup power</sub>

## Considerations for designing your grid
To guarantee grid stability, Ficsit advises to have a minimum of one power storage connected
directly to the grid. This ensures there will be enough power available for the limited
amount of time it takes for your Backup Power Supply management system to flip its switches.

Please note that while the charge rate of a power storage is limited to 100 MW, its
discharge rate is unlimited. This means that for the maximum of five seconds it takes
for the BPS to provide backup power, a single power storage can supply as much
as 72,000 MW of power.

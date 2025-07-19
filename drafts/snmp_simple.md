# What is SNMP
Stands for **S**imple **N**etwork **M**anagement **P**rotocol. It's used to monitor and modify device settings through the network. It was originally designed for routers and switches, but has since been expanded to servers, printers, etc. An SNMP network comprises of three components
- SNMP Agent: Devices and it's software that provides status or can send (traps) to the SNMP manager.
- SNMP Manager (NMS): a central system that issues requests to agents, or receives traps from agents.
- OID: Object identifiers that label different systems, devices, and configurations that agents and managers read/modify from.

There are two ways that devices communicate via SNMP, polling and traps. Polling involves the NMS making requests to agents to configure or read the status of devices, while traps are notifications sent from an agent w/o any prior request. This article will mainly go over SNMP traps.

SNMP uses OID's (Object Identifiers) to specify what/which systems and configurations the SNMP request is giving the status on.

# OID
OID's are usually structured as a string of numbers - similar to an IP format.
Example OID's
- `1.3.6.1.2.1.1.1`
- `1.3.6.1.2.1.1.3`
- `1.3.6.1.4.1.29462.10.2.1.3.2.5.13.1`


# MIB
MIB stands for Management Information Base, and is a text file that provides a human readable string that associates with an OID. It's role is similar to host names - to make it easier to read what the SNMP request is altering. Usually SNMP managers will contain software to help translate OID's to MIB's (or vice versa), as well as installing pre-configured MIB files within a directory. It's common to have companies specifying a directory with multiple MIB's that pertain to their infrastructure devices or SAAS products.
# ![](/static/pictures/snmp_simple/mib_ex_1.png){.align-center} 

For example we can look at [Ingrasys](https://www.ingrasys.com/), a Taiwanese company that specializes in AI rack solutions and datacenters, has a MIB file for two different products.

# ![](/static/pictures/snmp_simple/ingrasys_usha_mib.png){.align-center} 
# ![](/static/pictures/snmp_simple/ingrasys_ipoman_mib.png){.align-center} 

We can see that the corresponding OID's are
- for USHA: 3.6.1.4.1.2468.1.2.1.1
- for IPOMANII: 3.6.1.4.1.2468.1.4.1

Additionally we can see that the prefix 3.6.1.4.1 specifies a private enterprise device for the internet. 2468 is the identifier for Ingrasys. The next subfix are product specifications created by the company through the MIB file.

We can also use the `snmptranslate` command to help parse MIB files to find the OID's.

Within the MIB files, we can see each number in the OID corresponds to a certain identifier, or segment. 
`iso(1) identified-organization(3) dod(6) internet(1) snmpV2(6) snmpModules(3) snmpAlarmNextIndex(1) snmpMIBObjects(1) snmpTraps(5)`

# SNMPTRAP command

The `snmptrap` command sends an notification to the NMS. It is an un-requested message that an agent sends to inform the manager about critical events (outages, downstate,enterprise alerts).

There are two types of traps, *generic traps* and *enterprise-specific traps* (in other words custom made).

## Generic Traps
We can pull from [RFC-1215](https://www.rfc-editor.org/rfc/rfc1215#section-2.2.2) a list of generic traps to use with SNMP. These are universally supported OID's that are supported through SNMP's protocol.

```
          coldStart TRAP-TYPE
              ENTERPRISE  snmp
              DESCRIPTION
                          "A coldStart trap signifies that the sending
                          protocol entity is reinitializing itself such
                          that the agent's configuration or the protocol
                          entity implementation may be altered."
              ::= 0

          warmStart TRAP-TYPE
              ENTERPRISE  snmp
              DESCRIPTION
                          "A warmStart trap signifies that the sending
                          protocol entity is reinitializing itself such
                          that neither the agent configuration nor the
                          protocol entity implementation is altered."
              ::= 1

          linkDown TRAP-TYPE
              ENTERPRISE  snmp
              VARIABLES   { ifIndex }
              DESCRIPTION
                          "A linkDown trap signifies that the sending
                          protocol entity recognizes a failure in one of
                          the communication links represented in the
                          agent's configuration."
              ::= 2

          linkUp TRAP-TYPE
              ENTERPRISE  snmp
              VARIABLES   { ifIndex }
              DESCRIPTION
                          "A linkUp trap signifies that the sending
                          protocol entity recognizes that one of the
                          communication links represented in the agent's
                          configuration has come up."
              ::= 3

          authenticationFailure TRAP-TYPE
              ENTERPRISE  snmp
              DESCRIPTION
                          "An authenticationFailure trap signifies that
                          the sending protocol entity is the addressee
                          of a protocol message that is not properly
                          authenticated.  While implementations of the
                          SNMP must be capable of generating this trap,
                          they must also be capable of suppressing the
                          emission of such traps via an implementation-
                          specific mechanism."
              ::= 4

          egpNeighborLoss TRAP-TYPE
              ENTERPRISE  snmp
              VARIABLES   { egpNeighAddr }
              DESCRIPTION
                          "An egpNeighborLoss trap signifies that an EGP
                          neighbor for whom the sending protocol entity
                          was an EGP peer has been marked down and the
                          peer relationship no longer obtains."
              ::= 5
```

You can also find these within your `.mib` directories of your SNMP manager. I'm using `librenms`, so they're found in `/opt/librenms/mibs/SNMPv2-MIB`.

We can also [lookup the generic traps OID](https://oidref.com/1.3.6.1.6.3.1.1.5.1), and see that the `::=1` line corresponds to the last number (snmpTraps) type.

## SNMP Trap Caveats

SNMP traps are un-requested messages sent by agents to notify the manager about critical events. However these messages are unacknowledged - they are sent over UDP. In theory they are unreliable because messages could be lost. However due to the nature of critical events - this is ok - because the device might not be able to wait for an acknowledgement (application or transport).
- UDP requires low overhead, so the impact on network is reduced.
- In heavily congested networks, SNMP over TCP causes more issues
- SNMP is used for monitoring - and needs to be able to work in unreliable networks.
"When a network is failing, a protocol that tries to get the data through but gives up if it can't is almost certainly a better design choice than a protocol that will flood the network with retransmissions in its attempt to achieve reliability." (1)[docstore.mik.ua/orelly/networking_2ndEd/snmp/ch02_01.html]

It is possible to have SNMP traps to be sent over (TCP)[http://www.faqs.org/rfcs/rfc3430.html], but one must think carefully if it's needed. TCP will only provide delivery guarantees for layer 3, and comes at a cost of latency and retransmissions. The agent and server have to wait for an acknowledgement from each other. The better guarantee is using SNMP informs.


## snmptrap command structure

You'll need to install `snmp` and `snmptrapd` in order to use the `snmptrap` command.

Before getting to the NMS configuration we'll go over the `snmptrap` command structure.

```
snmptrap -v <snmp_version> -c <community> <destination_host> <uptime> <OID_or_MIB> <object> <value_type> <value>
```
Example:
`snmptrap -v 2c -c public localhost '' 1.3.6.1.6.3.1.1.5 1.3.6.1.6.3.1.1.5.1 s "This is a generic trap about the cold start"`

Let's break this command down:
- `-v`: this specifies the version of SNMP to use. Values can be `1,2c,3`. We'll go over the differences later in the article.
- `-c`: this specifies community string. A community string is created by the NMS, and agents need to use the community string to be able to send and receive requests from the NMS. It's worth noting it is **not encrypted**.
- `<community>=public`: the default community string
- `<destination_host>=localhost`: ip or host of the NMS server.
- `<uptime>=''`: Easily the most confusing part of the command. Every trap needs to specify the uptime (of the system, device, server, something). Specifying '' instead of the OID and value will use the system's `host-time`
- `<OID_or_MIB>`: This sepcifies the OID or MIB
- `<object> <value_type> <value>`: We can specify multiple OID/MIB onto the snmp trap. In the example `1.3.6.1.6.3.1.1.5.1 s "This is a generic trap about the cold start"`, we've specified the coldstart OID, with the value type of a string, and the value message.

# SNMP Inform

The issue with TRAP's is that they are inherently unreliable. An extra step is an application (SNMP) acknowledgement - an `snmpinform` PDU. `snmpinform` is just an acknowledged `snmptrap`, when the manager receives an inform it will respond with a message acknowledging the trap.

# ![](/static/pictures/snmp_simple/wireshark_inform_3.png){.align-center} 
With these two inform requests, one with an invalid community string, we can open up Wireshark to see what's going on in the background.

# ![](/static/pictures/snmp_simple/wireshark_inform_2.png){.align-center} 
We can see in this picture that the SNMP protocol was acknowledged with a SNMP `get-reponse`. The failed SNMP inform request was retried at a 1 second interval.

>It's also worth noting that our community string is not encrypted.
# ![](/static/pictures/snmp_simple/wireshark_community_0.png){.align-center} 
Usually SNMP is implemented within a company subnet, but if it needs to span into the public domain, or cross platforms - it will be insecure because packet data can be sniffed and saved. Additionally bad actors can use `inform` to see if a particular community strong gets through.


# Securing SNMP Trap

SNMP versions 1 and 2c use a community string for authentication. However they are vulnerable because they are sent with clear text. This is troublesome because anyone snooping on the network will be able to find out what the community string is. It's important to use a strong community string and also alter read-write controls for certain community strings. If possible use read-only access for community strings.

## snmptrapd
This is an SNMP server for listening to TRAP and INFORM messages from agents. It listens on port 162.
Documentation for altering the configuration can be found (here)[https://www.net-snmp.org/docs/man/snmptrapd.conf.html].

It is imperative when using 1 and 2c that access control is set up on the server.

*authCommunity* is a setting that authorizes traps and informs to trigger the types of processing listed, and only work on the specified community strings. Additionally you can further restrict it to certain OID's.

## SNMP v3

SNMP version 3 comes with a whole slew of security improvements. It gets rid of the community string, and instead opts for a username and password authentication protocol - making away with the unencrypted community password. SNMP v3 allows the SNMP Trap server to also set access controls to users.

To enable SNMPv3, in `/etc/snmp/snmptrapd.conf` you'll have create a user. 

### Configure SNMPv3 User

`createUser -e ENGINEID username (MD5|SHA) authpassphrase [DES|AES] `

- **ENGINEID:** the EngineID of the application that is going to be sending the trap. This is used for identifying an SNMPv3 entity (agent, manager, device, etc). This helps ensuring that the manager (receiver of the trap) expects the correct engine ID in order for the trap to be processed.

- **myuser:** the username that is going to be sending the trap

- **SHA:** the authentication type (SHA or MD5, with SHA being better)

- **"my authentication pass":** The authentication pass-phrase to use to generate the secret authentication key

- **"AES"**: encryption type to use (AES or DES)

- **"my encryption pass"**: The encryption pass-phrase to use to generate the secret encyrption key. If it's blank it will use the same password as SHA

`authUser TYPES [-s MODEL] USER [LEVEL [OID | -v VIEW ]]`
This provides what type of processing the trap daemon will do when it receives the trap notification.
- **TYPES:** The available types are `log,execute,net`.
- **LEVEL:** This field can be used to allow unauthenticated notifications (noAuth). If not set it's defaulted to *authPriv*

Example Configuration:
```
createUser -e 0x80001F88C0A880 labUserTrap SHA mavlabpass123 AES mavlabpass123
authUser log,execute,net labUserTrap authPriv
```

Using the `snmptrap` command, we can create a request to the trapd server.

`snmptrap -v 3 -n "" -a SHA -A mavlabpass123 -x AES -X mavlabpass123 -l authPriv -u labUserTrap -e 0x80001F88C0A880 192.168.1.67 '' 1.3.6.1.4.1.2468.1.2.1.1 1.3.6.1.6.3.1.1.5.1 s "Trap over SNMPv3`

# ![](/static/pictures/snmp_simple/snmp_3_log.png){.align-center} 

# ![](/static/pictures/snmp_simple/snmp_3_wireshark.png){.align-center} 
> The username and password is not leaked

---
date: 07/15/2025
title: Securing SNMP with VACM (View Access Control Models)
blurb: Further secure SNMP devices by setting VACM for both 2c and version 3.
---

Net-SNMP allows SNMP agents to configure View Access Control Models (VACM) to restrict what MIB's/OID's can read, set, or notify - effectively setting access control for users. Agents that don't specify these views are vulnerable to leaking critical information about their system - especially on version 2c or below. Additionally views can help with protecting what OID's/MIB's can be written to or notified (TRAP) to, preventing misconfigurations or development mistakes. 


# SNMP1 and SNMP2 Access Control

Use of SNMP2c and below is highly discouraged, as all packets are in plaintext. It's trivial to snoop on SNMPv2 packets as shown in the `SNMP_TRAP` article. Reason being - all you need is the correct community string to view MIB information - which can be retrieved from the SNMP packet.

Before getting to  VACM, we should go over traditional access control. SNMP offers traditional access control that restricts the hosts or subnets that could use a particular community string, as well as *readyonly* and *readwrite* permissions.

- rocommunity COMMUNITY [SOURCE [OID | -V VIEW [CONTEXT]]]
- rwcommunity COMMUNITY [SOURCE [OID | -V VIEW [CONTEXT]]]

### Traditional ACL Example
In our `snmpd.conf` we can configure a community string to only be able to be used within an internal IP subnet.
```
rocommunity LABINTERNAL 192.168.1.0/24
```
# ![](/static/pictures/snmp_acl/snmp_acl_get_v2.png){.align-center} 

We can see that from our internal subnet - we were able to read the device location, however we're unable to alter that OID with the `snmpset` command

Mis-configuring `com2sec internal 192.168.1.0/24 LABINTERNAL` to `com2sec internal 192.168.50.0/24 LABINTERNAL` will cause nodes to be unable to access the MIB tree with that community string.

## Configuring VACM for SNMP1 and SNMP2

VACM makes it easier to configure access control for more complicated rule sets, allowing different views to be set for certain types of requests (GET vs SET), more flexible access control configurations, and evening more flexibility for OID matching.

Let's look a the configuration options used to configure VACM.

It's possible to restrict SNMPv1 and SNMPv2 community strings to certain IP's. This is highly recommended if using those versions. The settings are
- com2sec  SECNAME SOURCE COMMUNITY
- com2sec6  SECNAME SOURCE COMMUNITY
- com2secunix SECNAME SOCKPATH COMMUNITY

SECNAME refers to the name of this security (view) model.<br>
SOURCE refers to a hostname or a range of IP(6) represented as an IP mask. *ie* 192.168.1.0/24 or 10.10.10.0/255.255.255.0<br>
COMMUNITY refers to the community string that this view is bound to.<br>
SOCKPATH is the unix domain socket path

Next let's create a group. A group allows configurations to map certain users or community strings together. It's the secret sauce to combining different security models to different views.

- group GROUP {v1|v2c|usm|tsm|ksm} SECNAME

GROUP refers to the group name<br>
{v1|v2c|usm|tsm|ksm} need to choose which version the mapping applies to. `usm,tsm,ksm` refer to different types of SNMP based algorithms.<br>
SECNAME refers to the username or community string that this group is applied to<br>


Views allow restrictions on the OID tree - restricting or allowing users to access parts of the OID.
- view VNAME TYPE OID [MASK]

VNAME refers to the name of the view<br>
TYPE can be `included` or `excluded`, giving flexibility to restrict parts of the OID tree users can access.<br>
OID can be an OID, *ie* .1 or .1.2.3, or a MIB *ie* .iso, .iso.org.dod.enterprise<br>
MASK is a list of hex octets (optionally separated by '.' or ':') with the set bits indicating which subidentifiers in the view OID to match against. If not specified, this defaults to matching the OID exactly (all bits set), thus defining a simple OID subtree. So:

```
view iso1 included .iso 0xf0
view iso2 included .iso
view iso3 included .iso.org.dod.mgmt 0xf0 
```

would all define the same view, covering the whole of the 'iso(1)' subtree (with the third example ignoring the subidentifiers not covered by the mask). 


Lastly we need the `access` setting to tie everything together.

`access GROUP CONTEXT {any|v1|v2c|usm|tsm|ksm} LEVEL PREFX READ WRITE NOTIFY`

- GROUP refers to the GROUP name we want to apply views to<br>
-{any|v1|v2c|usm|tsm|ksm} applies this model to a certain protocol<br>
- LEVEL is either `noauth,auth,priv`. Use `auth` to enforce authentication and encryption. `noauth` allows unauthenticated requests and `priv` only enforces encryption, no authentication.<br>
- PREFX determines how to match the VIEW's OID tree. Can be `includes,excludes`<br>
- READ WRITE NOTIFY applies views for GET,SET, and TRAP/INFORM requests. Can be se to `none` to restrict everything.<br>

## Example

Let's alter our `snmpd.conf` file again to apply the VACM to the LABINTERNAL community string.

```
#rocommunity LABINTERNAL
com2sec internal 192.168.1.0/24 LABINTERNAL

group LabInternal v2c internal

view labview included .1.3.6.1.2.1.1
access LabInternal "" any noauth exact labview labview labview
```

1. We comment out `rocommunity LABINTERNAL` because setting `rocommunity` will overwrite the VACM configurations that we set.
2. We've restricted the LABINTERNAL community string to an internal subnet using `com2sec`. It is equivalent to the `rocommunity` option.
3. Created a group LabInternal that operates on v2c connections, and maps it to the com2sec security model `internal`.
4. We've created a view that only allows requests to access .1.3.6.1.2.1.1
5. We've set the group to use that view for all requests on each of the GET,SET,NOTIFY operations.

Issuing an `snmpwalk` lists all MIB's from the input subtree.

`snmpwalk -v 2c -c LABINTERNAL 192.168.1.67 .1.3.6.1.2.1.1`

# ![](/static/pictures/snmp_acl/snmp_acl_walk_v2.png){.align-center} 
We can see that we're able to access this OID tree, and walk over the rest of the tree.

Moving up the tree results in the same output `snmpwalk -v 2c -c LABINTERNAL 192.168.1.67 .1.3.6.1.2.1`, because the view has restricted the full parent tree from being walked. Updating `view labview included .1.3.6.1.2.1.1` to `view labview excluded .1.3.6.1.2.1.1` will result on requests getting denied to view the whole OID subtree.



## Configuring VACM for SNMPv3

Setting up VACM for SNMPv3 is very similar to SNMP2c. All that's needed is to apply the group to the user, and then create the VACM using the `access` configuration option. When setting these settings, make sure to remove the `rouser` and `rwuser` options (similar to removing the `rocommunity` options for VACM).

To create users, you can specify the createUser directives directly in the configuration.
- createUser [-e ENGINEID] username (MD5|SHA) authpassphrase [DES|AES] [privpassphrase]

or you can run the `net-snmp-create-v3-user` command to create them. The users data is then stored in `/var/lib/snmpd.conf`. In this file is where you can extract out the engineID's to use in traps, etc.

### Example
```
createUser qplabmav SHA qplabpassword AES qplabpassword

# specify a readonly group
group QPLABv3Group usm qplabmav

# view that restricts to oid's that start with .1
view v3Uptime included .1.3.6.1.2.1.1.3
access QPLABv3Group "" any auth exact v3Uptime none none
```

After extracting the engine ID from `/var/lib/snmpd.conf`, we can use it to issue an `snmpget` to obtain the uptime of the snmpd server.
# ![](/static/pictures/snmp_acl/snmp_acl_get_v3.png){.align-center} 

As you can see, the user is only able to obtain information from only the Uptime OID, and nothing else.


# Conclusion

ACL's adds another layer to security to SNMP, and is recommended when using it in production environments. If using SNMPv1 or SNMPv2 - it's incredibly important to at least set some form of access controls onto the system - or else an attacker can use `snmpwalk` to find critical information about the system - or worse alter the values.
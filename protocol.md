## message format
* u8 opcode
* u64 request-id
* varlen length
* data

In requests, the client will provide a request ID. The Server MUST set the exact same request ID in a response.
A client MAY reuse a reuse a request ID after all required responses for the original request were received.

Any responses by the server must use the exact same request ID as the corresponding request. request-id 0 is reserved for responses not associated with any request

## opcode to server
* 0 - hello
* 1 - get devices
* 2 - set channel
* 3 - get channel
* 4 - subscribe channel

## opcode to client
* 1 - welcome
* 2 - devices
* 3 - channel value
* 4 - ok
* 5 - err

## data structures

### varlen
* 1 bit - large string
* 7 bit / 15 bit (if large) - length

the size of the length field is never included in the length.

if anything is longer than expected, extra bytes are simply ignored. If length is shorter than expected, missing fields should be filled with sane defaults. If this is impossible, an error should be produced

### string
* varlen size
* x byte data

### room descriptor
* varlen size
* string id
* string display_name

### channel descriptor
* varlen size
* u8 flags
    * 0x01 - subscribe
    * 0x02 - write
    * 0x04 - read
    * 0x08 - linger
* bytes room
* string display_name
* u8 type
    * 0 - boolean
    * 1 - u8
    * 2 - u32
    * 3 - f32
    * 4 - RGB
    * 5 - event
    * 6 - enum
        * varlen enum_values
        * x enum_values
            * string display_name
    * 7 - UTF8 String
    * 8 - binary
    * 9 - cbor encoded data
* u8 kind
`   * 0 - other
    * 1 - lamp
    * 2 - wall socket
    * 3 - relay /contactor
    * 4 - temperature sensor
    * 5 - button
    * 6 - volume
    * 7 - borg / LED Matrix


### device descriptor
* varlen size
* bytes id
* string display_name
* string wiki_url
* varlen channel_count
* x channel descriptors

### RGB
* u8 red
* u8 green
* u8 blue

### binary
* varlen length
* x byte data

## messages

### hello

### welcome

### get_devices
_no payload_

### devices
* u16 room_count
* x room descriptor
* u16 device_count
* x device descriptor

### set channel
* string device_id
* string room_id
* string channel_id
* varlen data_len
* data

### get channel
* string device_id
* string room_id
* string channel_id

### channel value
* u8 flags
    * 0x01 - linger - if set the value has been cached
* varlen data_len
* data

### subscribe channel
* string device_id
* string room_id
* string channel_id

### channel event
* varlen data_len
* data

Any change of the channel value. For event type channels, the length is 0 and there is no payload.

### ok
_empty payload_
States that the request was successfully received, but did not produce any output.

For request of type subscribe this means that the subscription has been received. Events on that subscription will reuse the request-id
For any other type this means the request ID can be reused.

### err
u16 machine readable error code
* 0 - device does not exist
* 1 - room does not exist
* 2 - channel does not exist
* 3 - invalid request for channel
* 4 - value unknown - means a linger channel has not received an update yet
string human readable string

If request-id is 0, this message indicates an error affecting the whole connection. The connection MUST be closed after this.
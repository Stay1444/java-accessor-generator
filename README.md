# Java Accessor Generator

Generates java mappings to access objects you don't have access to, using reflection.

Turns files like these
```ron
(
    name: "MovementPacket",
    package: "dev.local.Accessors",
    fields: [
        ( name: "X", type: i32 ),
        ( name: "Y", type: i32 ),
    ]
)
```
Into these (simplified)
```java
package dev.local.Accessors;

public class MovementPacketAccessor {
    public int X;
    public int Y;

    public void setX();
    public void setY();

    public static MovementPacketAccessor access(Object object);
}
```
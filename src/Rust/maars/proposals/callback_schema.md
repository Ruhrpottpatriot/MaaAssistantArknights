# RFC: Callback Schema Stabilization

## Summary

Currently the callback schema is in flux. This document aims to provide a starting point for discussion a stabilization of said schema. Having a well-defined and stable callback messaging scheme helps developers that want to integrate MAACore into their application. It avoid pitfalls by providing ample documentation and helps in keeping code backwards-compatible.

## Proposal

The current callback schema, as described in **3.2-CALLBACK_SCHEMA** presents the user with the following prototype for the callback function:

```C
typedef void (ASST_CALL* AsstCallback)(
  int msg,
  const char* details,
  *void custom_arg
);
```

In that callback, the parameter `msg` is the type of the message, the parameter details` is the actual message encoded as a JSON struct, and the `custom_arg` parameter is any additional extra data that might be needed in the callback. For example _C#_ and _Rust_ might use this to pass a `this`/`self` pointer. While this declaration is not totally unsuited for the messaging task, it is somewhat limited in what it can provide.

Therefore, this RFC proposes to change the messaging interface to resemble structured logging. By going this route the interface can be stabilized without hindering the possibility to add new message types later on. The change is relatively small, since the actual message details are already expected to be JSON and the main change for the interface would be to move the message id/message type parameter into the actual message. A (theoretical) message would then look like this:

```json
{
  "msg": string,
  "type": string,
  "details": {
    // Additional fields
  }
}
```

In the above JSON holds the message type in the `msg_type` as `string`, opposed to the current `int` type. This change doesn't makes parsing not much more complicated, as languages can either use `if/else` or `match` statements to convert the type string into an enum, or their language equivalent. The second change is to have a dedicated `msg` field inside the JSON that holds a _short_ (about 50 to 72 characters) summary, whose main intention is to be displayed to the user. The reason for a short length is because the actual details should be held in the `details` map with dedicated fields.

Approaching the message API in a structured way, makes it possible to attach additional common information fields. For now, these should be the timestamp the message was created, formatted in ISO-8601. ISO-8601 was chosen because, as opposed to UNIX timestamps, it's human readable and supports timezones. The next common parameters added is a globally unique id for the message, based on RFC4122 Version 4. With these additional fields, the
message structure would look like this:

```json
{
  "id": uuid,
  "timestamp": string,
  "msg": string,
  "type": string,
  "details": {
    // Additional fields
  }  
}
```

The above JSON message structure presents a base framework, that can easily be extended with additional data. This information is added to the `details` field following the same logic as adding the details itself. This approach yields a nested structure that can easily be parsed and transformed if needed. For example, looking at the _AsyncCallInfo_ message type from chapter **3.2**, which has two detail fields: `ret` holding the boolean return value indicating success or failure, and `cost` holding the duration in milliseconds the async call ran. By adding these fields to the `details` field would produce a message that might look like this:

```json
{
  "id": "b3c0f9e1-52a7-4d6c-8e3f-1ca8aa0c95ef",
  "timestamp": "2024-10-04T10:13:47+0000",
  "msg": "Async call completed",
  "type": "AsyncCallInfo",
  "details": {
    "ret": true,
    "cost": 600
  }
}

```
 
However, the above task is very simple. It has only two fields and no nested information that needs to be conveyed. That said, if (deeply) nested data is needed, the proposed changes can account for this. Nested data is represented as adjacently tagged structures, where the outer `details` field has an inner `details` field, whose type is tagged by the nested `type` field, i.e.

```json
{
  "id": uuid,
  "timestamp": string,
  "msg": string,
  "type": string,
  "details": {
    "type": string,
    "details": {
      // Additional fields
    }
  }
```

This approach allows parsing of arbitrary nesting of data. For example, below is how a `SubTaskStart` message could look like:

```json
{
  "id": "34d021dc-0231-4d5b-b4c5-61da8639be1a",
  "timestamp": "2024-08-14T20:47:29+0000",
  "msg": "Received information about a subtask",
  "type": "SubTaskStart",
  "details": {
    "type": "ProcessTask",
    "details":  {
      "task_id": "StartButton2",
      "action": 512,
      "exec_times": 1,
      "max_times": 999,
      "algorithm": 0,
    }
  }
}
```

An even more nested structure could then look like this:

```json
{
  "id": "34d021dc-0231-4d5b-b4c5-61da8639be1a",
  "timestamp": "2024-08-14T20:47:29+0000",
  "msg": "Received information about a subtask",
  "type": "SubTaskExtraInfo",
  "details": {
    "taskchain": "Fight",
    "type": "StageDrops",
    "details": {
      "stage": {
        "stageCode": "CA-5",
        "stageId": "wk_fly_5"
      },
      "achievedRating": 3,
      "actualDrops": [
        {
          "itemId": "3301",
          "quantity": 2,
          "itemName": "技巧概要·卷1" // Skill Summary - 1
        },
        // ...
      ],
      "expectedDrops": [              // Statistics of drops
        {
            "itemId": "3301",
            "itemName": "技巧概要·卷1", // Skill Summary - 1
            "quantity": 4
        },
        // ...
      ]
    }
  }
}
```

While the above example could be simplified by removing some nesting, it shows how a complex message could be passed in a a structured way that is easy to parse into a user facing string.


## Drawbacks

While the overall change to the callback schema isn't too big, it still necessitates changes to the function signature, as well as the field names inside the JSON message struct, and therefore is a breaking change in the API-contract. This might hinder adoption of the proposal.
An alternative would be to _not_ change the messaging API and improve.
However, at some point the API should be stabilized and it's better to do so as early as possible. The changes in the signature also _might_ lead to a slight regression in performance, as comparing strings is always slower than comparing two numbers. That said, the impact on the user should be fairly minimal.

Implementations that don't support proper sum types (also called tagged unions) might need to put in more work when (de-)serializing the message string into a language type. However, this is also the case when message type and data are separated into two distinct field. On the other hand, making the structure more regular helps with this task, since code can be reused more frequently.

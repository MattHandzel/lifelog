btw the interface should define a way to expliclity close the gRPC connection made with a request. So basically the gRPC stream will continue to send data until the connection is expliclity closed. You can expliclity close the connection when:

- The user does another query (it doesn't care about the old one)
- The buffer becomes full
- ...

So make a general function that when called expliclity closes the connection so the server stops processing the query

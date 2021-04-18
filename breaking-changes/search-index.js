var searchIndex = JSON.parse('{\
"hey_listen":{"doc":"<code>Hey_listen</code> is a collection of event-dispatchers aiming to …","t":[4,6,6,13,11,11,11,11,11,11,11,11,11,11,0,11,11,11,4,8,8,4,8,4,8,13,13,13,13,13,0,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,10,10,10,10,0,0,11,11,11,11,11,11,11,11,11,3,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,3,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,3,11,11,11,11,11,11,11,11,11,11,11,11,11,11],"n":["Error","Mutex","RwLock","ThreadPoolBuilder","borrow","borrow_mut","deref","deref_mut","drop","fmt","from","from","init","into","sync","try_from","try_into","type_id","AsyncDispatchResult","AsyncListener","Listener","ParallelDispatchResult","ParallelListener","PriorityDispatcherResult","PriorityListener","StopListening","StopListening","StopListening","StopListeningAndPropagation","StopPropagation","async_dispatcher","borrow","borrow","borrow","borrow_mut","borrow_mut","borrow_mut","deref","deref","deref","deref_mut","deref_mut","deref_mut","drop","drop","drop","fmt","fmt","fmt","from","from","from","init","init","init","into","into","into","on_event","on_event","on_event","on_event","parallel_dispatcher","priority_dispatcher","try_from","try_from","try_from","try_into","try_into","try_into","type_id","type_id","type_id","AsyncDispatcher","add_listener","borrow","borrow_mut","default","deref","deref_mut","dispatch_event","drop","from","init","into","new","try_from","try_into","type_id","ParallelDispatcher","add_listener","borrow","borrow_mut","deref","deref_mut","dispatch_event","drop","from","init","into","new","num_threads","try_from","try_into","type_id","PriorityDispatcher","add_listener","borrow","borrow_mut","default","deref","deref_mut","dispatch_event","drop","from","init","into","try_from","try_into","type_id"],"q":["hey_listen","","","","","","","","","","","","","","","","","","hey_listen::sync","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","hey_listen::sync::async_dispatcher","","","","","","","","","","","","","","","","hey_listen::sync::parallel_dispatcher","","","","","","","","","","","","","","","","hey_listen::sync::priority_dispatcher","","","","","","","","","","","","","",""],"d":["<code>hey_listen</code>’s Error collection. As long as there are no …","A mutual exclusion primitive useful for protecting shared …","A reader-writer lock","Error when building a threadpool fails.","","","","","","","","","","","The parallel/async dispatcher module.","","","","An <code>enum</code> returning a request from a [<code>Listener</code>] to its …","Every event-receiver needs to implement this trait in …","Every event-receiver needs to implement this trait in …","An <code>enum</code> returning a request from a <code>Listener</code> to its …","Every event-receiver needs to implement this trait in …","An <code>enum</code> returning a request from a listener to its <code>sync</code> …","Every event-receiver needs to implement this trait in …","Stops listening to the dispatcher.","Stops the listener from receiving further events from the …","Stops the listener from receiving further events from the …","Stops listening and prevents events from reaching the …","Prevents the event from reaching the next less important …","This module contains the async dispatcher.","","","","","","","","","","","","","","","","","","","","","","","","","","","","This function will be called once a listened event-type <code>T</code> …","This function will be called once a listened event-type <code>T</code> …","This function will be called once a listened event-type <code>T</code> …","This function will be called once a listened event-type <code>T</code> …","This module contains the parallel dispatcher.","This module contains the priority dispatcher.","","","","","","","","","","In charge of parallel dispatching to all listeners.","Adds a <code>AsyncListener</code> to listen for an <code>event_key</code>.","","","","","","All <code>AsyncListener</code>s listening to a passed <code>event_identifier</code> …","","","","","Create a new async dispatcher. Amount of threads must be …","","","","In charge of parallel dispatching to all listeners.","Adds a <code>ParallelListener</code> to listen for an <code>event_key</code>.","","","","","All <code>ParallelListener</code>s listening to a passed …","","","","","Creates a parallel dispatcher with <code>num_threads</code> amount of …","Immediately after calling this method, the dispatcher …","","","","In charge of prioritised sync dispatching to all …","Adds a <code>Listener</code> to listen for an <code>event_identifier</code>, …","","","","","","All <code>Listener</code>s listening to a passed <code>event_identifier</code> will …","","","","","","",""],"i":[0,0,0,1,1,1,1,1,1,1,1,1,1,1,0,1,1,1,0,0,0,0,0,0,0,2,3,4,2,2,0,2,3,4,2,3,4,2,3,4,2,3,4,2,3,4,2,3,4,2,3,4,2,3,4,2,3,4,5,6,7,8,0,0,2,3,4,2,3,4,2,3,4,0,9,9,9,9,9,9,9,9,9,9,9,9,9,9,9,0,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,0,11,11,11,11,11,11,11,11,11,11,11,11,11,11],"f":[null,null,null,null,[[]],[[]],[[["usize",15]]],[[["usize",15]]],[[["usize",15]]],[[["formatter",3]],["result",6]],[[["threadpoolbuilderror",3]]],[[]],[[],["usize",15]],[[]],null,[[],["result",4]],[[],["result",4]],[[],["typeid",3]],null,null,null,null,null,null,null,null,null,null,null,null,null,[[]],[[]],[[]],[[]],[[]],[[]],[[["usize",15]]],[[["usize",15]]],[[["usize",15]]],[[["usize",15]]],[[["usize",15]]],[[["usize",15]]],[[["usize",15]]],[[["usize",15]]],[[["usize",15]]],[[["formatter",3]],["result",6]],[[["formatter",3]],["result",6]],[[["formatter",3]],["result",6]],[[]],[[]],[[]],[[],["usize",15]],[[],["usize",15]],[[],["usize",15]],[[]],[[]],[[]],[[],[["option",4],["paralleldispatchresult",4]]],[[],[["option",4],["paralleldispatchresult",4]]],[[],[["prioritydispatcherresult",4],["option",4]]],[[],[["pin",3],["box",3]]],null,null,[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["typeid",3]],[[],["typeid",3]],[[],["typeid",3]],null,[[["sync",8],["asynclistener",8],["sized",8],["send",8]]],[[]],[[]],[[]],[[["usize",15]]],[[["usize",15]]],[[]],[[["usize",15]]],[[]],[[],["usize",15]],[[]],[[]],[[],["result",4]],[[],["result",4]],[[],["typeid",3]],null,[[["parallellistener",8],["send",8],["sync",8],["sized",8]]],[[]],[[]],[[["usize",15]]],[[["usize",15]]],[[]],[[["usize",15]]],[[]],[[],["usize",15]],[[]],[[["usize",15]],[["error",4],["result",4]]],[[["usize",15]],[["error",4],["result",4]]],[[],["result",4]],[[],["result",4]],[[],["typeid",3]],null,[[["prioritylistener",8],["send",8],["sync",8]]],[[]],[[]],[[]],[[["usize",15]]],[[["usize",15]]],[[]],[[["usize",15]]],[[]],[[],["usize",15]],[[]],[[],["result",4]],[[],["result",4]],[[],["typeid",3]]],"p":[[4,"Error"],[4,"PriorityDispatcherResult"],[4,"ParallelDispatchResult"],[4,"AsyncDispatchResult"],[8,"Listener"],[8,"ParallelListener"],[8,"PriorityListener"],[8,"AsyncListener"],[3,"AsyncDispatcher"],[3,"ParallelDispatcher"],[3,"PriorityDispatcher"]]}\
}');
if (window.initSearch) {window.initSearch(searchIndex)};
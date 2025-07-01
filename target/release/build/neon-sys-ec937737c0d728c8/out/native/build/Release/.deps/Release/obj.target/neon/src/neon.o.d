cmd_Release/obj.target/neon/src/neon.o := c++ -o Release/obj.target/neon/src/neon.o ../src/neon.cc '-DNODE_GYP_MODULE_NAME=neon' '-DUSING_UV_SHARED=1' '-DUSING_V8_SHARED=1' '-DV8_DEPRECATION_WARNINGS=1' '-DV8_DEPRECATION_WARNINGS' '-DV8_IMMINENT_DEPRECATION_WARNINGS' '-D_GLIBCXX_USE_CXX11_ABI=1' '-D_DARWIN_USE_64_BIT_INODE=1' '-D_LARGEFILE_SOURCE' '-D_FILE_OFFSET_BITS=64' '-DOPENSSL_NO_PINSHARED' '-DOPENSSL_THREADS' '-DBUILDING_NODE_EXTENSION' -I/Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node -I/Users/xaxpmore/Library/Caches/node-gyp/18.20.5/src -I/Users/xaxpmore/Library/Caches/node-gyp/18.20.5/deps/openssl/config -I/Users/xaxpmore/Library/Caches/node-gyp/18.20.5/deps/openssl/openssl/include -I/Users/xaxpmore/Library/Caches/node-gyp/18.20.5/deps/uv/include -I/Users/xaxpmore/Library/Caches/node-gyp/18.20.5/deps/zlib -I/Users/xaxpmore/Library/Caches/node-gyp/18.20.5/deps/v8/include -I../node_modules/nan  -O3 -gdwarf-2 -mmacosx-version-min=10.15 -arch arm64 -Wall -Wendif-labels -W -Wno-unused-parameter -std=gnu++17 -stdlib=libc++ -fno-rtti -fno-exceptions -fno-strict-aliasing -MMD -MF ./Release/.deps/Release/obj.target/neon/src/neon.o.d.raw   -c
Release/obj.target/neon/src/neon.o: ../src/neon.cc \
  ../node_modules/nan/nan.h \
  /Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/node_version.h \
  /Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/uv.h \
  /Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/uv/errno.h \
  /Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/uv/version.h \
  /Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/uv/unix.h \
  /Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/uv/threadpool.h \
  /Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/uv/darwin.h \
  /Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/node.h \
  /Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8.h \
  /Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/cppgc/common.h \
  /Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8config.h \
  /Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-array-buffer.h \
  /Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-local-handle.h \
  /Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-internal.h \
  /Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-version.h \
  /Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-object.h \
  /Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-maybe.h \
  /Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-persistent-handle.h \
  /Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-weak-callback-info.h \
  /Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-primitive.h \
  /Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-data.h \
  /Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-value.h \
  /Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-traced-handle.h \
  /Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-container.h \
  /Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-context.h \
  /Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-snapshot.h \
  /Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-date.h \
  /Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-debug.h \
  /Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-script.h \
  /Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-message.h \
  /Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-exception.h \
  /Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-extension.h \
  /Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-external.h \
  /Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-function.h \
  /Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-function-callback.h \
  /Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-template.h \
  /Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-memory-span.h \
  /Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-initialization.h \
  /Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-callbacks.h \
  /Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-isolate.h \
  /Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-embedder-heap.h \
  /Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-microtask.h \
  /Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-statistics.h \
  /Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-promise.h \
  /Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-unwinder.h \
  /Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-embedder-state-scope.h \
  /Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-platform.h \
  /Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-json.h \
  /Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-locker.h \
  /Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-microtask-queue.h \
  /Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-primitive-object.h \
  /Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-proxy.h \
  /Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-regexp.h \
  /Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-typed-array.h \
  /Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-value-serializer.h \
  /Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-wasm.h \
  /Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/node_api.h \
  /Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/js_native_api.h \
  /Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/js_native_api_types.h \
  /Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/node_api_types.h \
  /Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/node_buffer.h \
  /Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/node_object_wrap.h \
  ../node_modules/nan/nan_callbacks.h \
  ../node_modules/nan/nan_callbacks_12_inl.h \
  ../node_modules/nan/nan_maybe_43_inl.h \
  ../node_modules/nan/nan_converters.h \
  ../node_modules/nan/nan_converters_43_inl.h \
  ../node_modules/nan/nan_new.h \
  ../node_modules/nan/nan_implementation_12_inl.h \
  ../node_modules/nan/nan_persistent_12_inl.h \
  ../node_modules/nan/nan_weak.h ../node_modules/nan/nan_object_wrap.h \
  ../node_modules/nan/nan_private.h \
  ../node_modules/nan/nan_typedarray_contents.h \
  ../node_modules/nan/nan_json.h ../node_modules/nan/nan_scriptorigin.h \
  ../src/neon.h ../src/neon_string.h ../src/neon_class_metadata.h \
  ../src/neon_task.h ../src/neon_event.h
../src/neon.cc:
../node_modules/nan/nan.h:
/Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/node_version.h:
/Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/uv.h:
/Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/uv/errno.h:
/Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/uv/version.h:
/Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/uv/unix.h:
/Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/uv/threadpool.h:
/Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/uv/darwin.h:
/Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/node.h:
/Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8.h:
/Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/cppgc/common.h:
/Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8config.h:
/Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-array-buffer.h:
/Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-local-handle.h:
/Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-internal.h:
/Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-version.h:
/Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-object.h:
/Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-maybe.h:
/Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-persistent-handle.h:
/Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-weak-callback-info.h:
/Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-primitive.h:
/Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-data.h:
/Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-value.h:
/Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-traced-handle.h:
/Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-container.h:
/Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-context.h:
/Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-snapshot.h:
/Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-date.h:
/Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-debug.h:
/Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-script.h:
/Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-message.h:
/Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-exception.h:
/Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-extension.h:
/Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-external.h:
/Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-function.h:
/Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-function-callback.h:
/Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-template.h:
/Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-memory-span.h:
/Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-initialization.h:
/Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-callbacks.h:
/Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-isolate.h:
/Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-embedder-heap.h:
/Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-microtask.h:
/Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-statistics.h:
/Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-promise.h:
/Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-unwinder.h:
/Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-embedder-state-scope.h:
/Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-platform.h:
/Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-json.h:
/Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-locker.h:
/Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-microtask-queue.h:
/Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-primitive-object.h:
/Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-proxy.h:
/Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-regexp.h:
/Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-typed-array.h:
/Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-value-serializer.h:
/Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/v8-wasm.h:
/Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/node_api.h:
/Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/js_native_api.h:
/Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/js_native_api_types.h:
/Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/node_api_types.h:
/Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/node_buffer.h:
/Users/xaxpmore/Library/Caches/node-gyp/18.20.5/include/node/node_object_wrap.h:
../node_modules/nan/nan_callbacks.h:
../node_modules/nan/nan_callbacks_12_inl.h:
../node_modules/nan/nan_maybe_43_inl.h:
../node_modules/nan/nan_converters.h:
../node_modules/nan/nan_converters_43_inl.h:
../node_modules/nan/nan_new.h:
../node_modules/nan/nan_implementation_12_inl.h:
../node_modules/nan/nan_persistent_12_inl.h:
../node_modules/nan/nan_weak.h:
../node_modules/nan/nan_object_wrap.h:
../node_modules/nan/nan_private.h:
../node_modules/nan/nan_typedarray_contents.h:
../node_modules/nan/nan_json.h:
../node_modules/nan/nan_scriptorigin.h:
../src/neon.h:
../src/neon_string.h:
../src/neon_class_metadata.h:
../src/neon_task.h:
../src/neon_event.h:

import 'package:flame/components.dart';

class UiSignal {
  final String type;
  final Object? payload;

  const UiSignal(this.type, {this.payload});
}

mixin UiSignalListener on Component {
  void onUiSignal(UiSignal signal) {}
}

class UiSignalBus {
  final _listeners = <UiSignalListener>{};

  void register(UiSignalListener listener) => _listeners.add(listener);

  void unregister(UiSignalListener listener) => _listeners.remove(listener);

  void emit(UiSignal signal) {
    for (final listener in _listeners) {
      listener.onUiSignal(signal);
    }
  }
}

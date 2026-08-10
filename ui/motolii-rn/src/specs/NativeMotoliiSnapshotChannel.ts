import type {TurboModule} from 'react-native';
import {TurboModuleRegistry} from 'react-native';
import type {EventEmitter} from 'react-native/Libraries/Types/CodegenTypes';

export interface Spec extends TurboModule {
  readonly onSnapshotChanged: EventEmitter<string>;
}

export default TurboModuleRegistry.getEnforcing<Spec>(
  'MotoliiSnapshotChannel',
);

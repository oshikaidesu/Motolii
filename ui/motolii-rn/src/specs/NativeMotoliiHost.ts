import type {TurboModule} from 'react-native';
import {TurboModuleRegistry} from 'react-native';

export interface Spec extends TurboModule {
  dispatchIntent(intentJson: string): string;
  readSnapshot(): string;
}

export default TurboModuleRegistry.get<Spec>('NativeMotoliiHost');

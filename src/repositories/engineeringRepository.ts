import type { EngineeringCalibration, EngineeringCalibrationInput } from "../engineering/types";
import { invokeDatabase } from "./tauri";

export const engineeringRepository = {
  getCalibration: () => invokeDatabase<EngineeringCalibration>("get_engineering_calibration"),
  updateCalibration: (input: EngineeringCalibrationInput) => invokeDatabase<EngineeringCalibration>("update_engineering_calibration", { input }),
  resetCalibration: () => invokeDatabase<EngineeringCalibration>("reset_engineering_calibration"),
};

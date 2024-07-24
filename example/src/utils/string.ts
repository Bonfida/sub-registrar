import { Language, checkLanguage, findLang } from "@bonfida/emojis";

/**
 * Checks if a subdomain name is valid
 * @param string Subdomain name
 * @returns boolean indicating if subdomain is valid
 */
export const isValidSubdomain = (subdomain: string) => {
  if (subdomain.includes(".")) {
    return false;
  }

  const lang = findLang(subdomain);
  if (lang === Language.Unauthorized) {
    return false;
  }

  return checkLanguage(subdomain, lang);
};
